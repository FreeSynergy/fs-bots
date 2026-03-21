// bot-db — BotDb object and all bot domain entities.
//
// The single database object for the bot manager. All bot sub-crates depend
// on this crate — never on sqlx or sea-orm directly.
//
// Uses fs-db (SeaORM) for all persistence. The underlying database backend
// (SQLite, Postgres, …) is configured in fs-db — nothing here depends on it.
// To switch databases, only fs-db changes.

use chrono::Utc;
use fs_db::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait,
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Order, QuerySelect,
};
use fs_db::sea_orm::sea_query::Expr;
use anyhow::Result;

pub mod entities;

use crate::entities::{
    audit_log, bot_meta, child_bot, join_request, known_room,
    poll_state, room_collection, room_collection_member, subscription,
    sync_message, sync_rule,
};

const SCHEMA: &str = include_str!("../migrations/schema.sql");

// ── BotDb ─────────────────────────────────────────────────────────────────────

/// The bot manager's database handle.
///
/// Wraps a [`DatabaseConnection`] from `fs-db` and exposes typed repository
/// methods for every bot domain object. No raw SQL outside this file.
#[derive(Clone)]
pub struct BotDb {
    conn: DatabaseConnection,
}

impl BotDb {
    /// Open (or create) the database at `path` and apply the schema.
    pub async fn open(path: &str) -> Result<Self> {
        use fs_db::sea_orm::Database;
        let url = format!("sqlite://{}?mode=rwc", path);
        let conn = Database::connect(&url).await?;
        conn.execute_unprepared(SCHEMA).await?;
        Ok(Self { conn })
    }

    // ── Audit ─────────────────────────────────────────────────────────────────

    pub async fn audit(
        &self,
        actor_type: &str,
        actor_id: &str,
        platform: Option<&str>,
        room_id: Option<&str>,
        action: &str,
        target: Option<&str>,
        result: &str,
        detail: Option<&str>,
    ) -> Result<()> {
        audit_log::ActiveModel {
            actor_type: Set(actor_type.to_string()),
            actor_id:   Set(actor_id.to_string()),
            platform:   Set(platform.map(str::to_string)),
            room_id:    Set(room_id.map(str::to_string)),
            action:     Set(action.to_string()),
            target:     Set(target.map(str::to_string)),
            result:     Set(result.to_string()),
            detail:     Set(detail.map(str::to_string)),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(())
    }

    pub async fn recent_audit(&self, limit: u64) -> Result<Vec<audit_log::Model>> {
        Ok(audit_log::Entity::find()
            .order_by(audit_log::Column::Id, Order::Desc)
            .limit(limit)
            .all(&self.conn)
            .await?)
    }

    // ── Poll state ────────────────────────────────────────────────────────────

    pub async fn get_offset(&self, platform: &str, room_id: &str) -> Result<u64> {
        let row = poll_state::Entity::find_by_id((platform.to_string(), room_id.to_string()))
            .one(&self.conn)
            .await?;
        Ok(row.map(|r| r.last_offset as u64).unwrap_or(0))
    }

    pub async fn set_offset(&self, platform: &str, room_id: &str, offset: u64) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        let model = poll_state::ActiveModel {
            platform:    Set(platform.to_string()),
            room_id:     Set(room_id.to_string()),
            last_offset: Set(offset as i64),
        };
        poll_state::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([poll_state::Column::Platform, poll_state::Column::RoomId])
                    .update_column(poll_state::Column::LastOffset)
                    .to_owned(),
            )
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    // ── Known rooms ───────────────────────────────────────────────────────────

    pub async fn upsert_room(
        &self,
        platform: &str,
        room_id: &str,
        room_name: Option<&str>,
        member_count: Option<i64>,
    ) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        let model = known_room::ActiveModel {
            platform:     Set(platform.to_string()),
            room_id:      Set(room_id.to_string()),
            room_name:    Set(room_name.map(str::to_string)),
            member_count: Set(member_count),
            last_seen:    Set(Utc::now().to_rfc3339()),
            ..Default::default()
        };
        known_room::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([known_room::Column::Platform, known_room::Column::RoomId])
                    .update_columns([
                        known_room::Column::RoomName,
                        known_room::Column::MemberCount,
                        known_room::Column::LastSeen,
                    ])
                    .to_owned(),
            )
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    pub async fn filter_rooms(&self, filter: &GroupFilter) -> Result<Vec<known_room::Model>> {
        let mut query = known_room::Entity::find();
        if let Some(ref platform) = filter.platform {
            query = query.filter(known_room::Column::Platform.eq(platform.as_str()));
        }
        if let Some(ref name) = filter.name_contains {
            query = query.filter(known_room::Column::RoomName.contains(name.as_str()));
        }
        if let Some(min) = filter.min_members {
            query = query.filter(known_room::Column::MemberCount.gte(min));
        }
        if let Some(max) = filter.max_members {
            query = query.filter(known_room::Column::MemberCount.lte(max));
        }
        Ok(query.order_by_asc(known_room::Column::RoomName).all(&self.conn).await?)
    }

    // ── Subscriptions ─────────────────────────────────────────────────────────

    pub async fn subscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        let model = subscription::ActiveModel {
            platform:   Set(platform.to_string()),
            room_id:    Set(room_id.to_string()),
            topic:      Set(topic.to_string()),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        };
        subscription::Entity::insert(model)
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    pub async fn unsubscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        subscription::Entity::delete_many()
            .filter(subscription::Column::Platform.eq(platform))
            .filter(subscription::Column::RoomId.eq(room_id))
            .filter(subscription::Column::Topic.eq(topic))
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    pub async fn subscriptions_for_room(&self, platform: &str, room_id: &str) -> Result<Vec<String>> {
        Ok(subscription::Entity::find()
            .filter(subscription::Column::Platform.eq(platform))
            .filter(subscription::Column::RoomId.eq(room_id))
            .all(&self.conn)
            .await?
            .into_iter()
            .map(|r| r.topic)
            .collect())
    }

    /// All (platform, room_id) pairs subscribed to the given topic.
    pub async fn subscriptions_for_room_by_topic(&self, topic: &str) -> Result<Vec<(String, String)>> {
        Ok(subscription::Entity::find()
            .filter(subscription::Column::Topic.eq(topic))
            .all(&self.conn)
            .await?
            .into_iter()
            .map(|r| (r.platform, r.room_id))
            .collect())
    }

    // ── Room collections ──────────────────────────────────────────────────────

    pub async fn create_collection(&self, name: &str, description: Option<&str>) -> Result<i64> {
        let result = room_collection::ActiveModel {
            name:        Set(name.to_string()),
            description: Set(description.map(str::to_string)),
            created_at:  Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(result.id)
    }

    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        room_collection::Entity::delete_by_id(id).exec(&self.conn).await?;
        Ok(())
    }

    pub async fn list_collections(&self) -> Result<Vec<room_collection::Model>> {
        Ok(room_collection::Entity::find()
            .order_by_asc(room_collection::Column::Name)
            .all(&self.conn)
            .await?)
    }

    pub async fn add_to_collection(&self, collection_id: i64, platform: &str, room_id: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        room_collection_member::Entity::insert(room_collection_member::ActiveModel {
            collection_id: Set(collection_id),
            platform:      Set(platform.to_string()),
            room_id:       Set(room_id.to_string()),
        })
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(&self.conn)
        .await?;
        Ok(())
    }

    pub async fn remove_from_collection(&self, collection_id: i64, platform: &str, room_id: &str) -> Result<()> {
        room_collection_member::Entity::delete_many()
            .filter(room_collection_member::Column::CollectionId.eq(collection_id))
            .filter(room_collection_member::Column::Platform.eq(platform))
            .filter(room_collection_member::Column::RoomId.eq(room_id))
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    pub async fn rooms_in_collection(&self, collection_id: i64) -> Result<Vec<room_collection_member::Model>> {
        Ok(room_collection_member::Entity::find()
            .filter(room_collection_member::Column::CollectionId.eq(collection_id))
            .all(&self.conn)
            .await?)
    }

    // ── Join requests ─────────────────────────────────────────────────────────

    pub async fn add_join_request(&self, platform: &str, room_id: &str, user_id: &str) -> Result<i64> {
        let result = join_request::ActiveModel {
            platform:   Set(platform.to_string()),
            room_id:    Set(room_id.to_string()),
            user_id:    Set(user_id.to_string()),
            status:     Set("pending".to_string()),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(result.id)
    }

    pub async fn get_join_request(&self, id: i64) -> Result<Option<join_request::Model>> {
        Ok(join_request::Entity::find_by_id(id).one(&self.conn).await?)
    }

    pub async fn list_pending_join_requests(&self, platform: &str, room_id: &str) -> Result<Vec<join_request::Model>> {
        Ok(join_request::Entity::find()
            .filter(join_request::Column::Platform.eq(platform))
            .filter(join_request::Column::RoomId.eq(room_id))
            .filter(join_request::Column::Status.eq("pending"))
            .order_by_asc(join_request::Column::CreatedAt)
            .all(&self.conn)
            .await?)
    }

    pub async fn resolve_join_request(&self, id: i64, status: &str, iam_result: Option<&str>) -> Result<()> {
        let mut model: join_request::ActiveModel = join_request::Entity::find_by_id(id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("join request {id} not found"))?
            .into();
        model.status = Set(status.to_string());
        model.iam_result = Set(iam_result.map(str::to_string));
        model.resolved_at = Set(Some(Utc::now().to_rfc3339()));
        model.update(&self.conn).await?;
        Ok(())
    }

    // ── Child bots ────────────────────────────────────────────────────────────

    pub async fn add_child_bot(&self, name: &str, bot_type: &str, data_dir: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        child_bot::Entity::insert(child_bot::ActiveModel {
            name:       Set(name.to_string()),
            bot_type:   Set(bot_type.to_string()),
            data_dir:   Set(data_dir.to_string()),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        })
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(&self.conn)
        .await?;
        Ok(())
    }

    pub async fn list_child_bots(&self) -> Result<Vec<child_bot::Model>> {
        Ok(child_bot::Entity::find()
            .order_by_asc(child_bot::Column::Name)
            .all(&self.conn)
            .await?)
    }

    pub async fn set_child_bot_status(&self, name: &str, status: &str, pid: Option<i64>) -> Result<()> {
        let mut model: child_bot::ActiveModel = child_bot::Entity::find()
            .filter(child_bot::Column::Name.eq(name))
            .one(&self.conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("child bot '{name}' not found"))?
            .into();
        model.status = Set(status.to_string());
        model.pid = Set(pid);
        model.update(&self.conn).await?;
        Ok(())
    }

    // ── Bot meta ──────────────────────────────────────────────────────────────

    pub async fn get_meta(&self, key: &str) -> Result<Option<String>> {
        Ok(bot_meta::Entity::find_by_id(key.to_string())
            .one(&self.conn)
            .await?
            .map(|m| m.value))
    }

    pub async fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        bot_meta::Entity::insert(bot_meta::ActiveModel {
            key:   Set(key.to_string()),
            value: Set(value.to_string()),
        })
        .on_conflict(
            OnConflict::column(bot_meta::Column::Key)
                .update_column(bot_meta::Column::Value)
                .to_owned(),
        )
        .exec(&self.conn)
        .await?;
        Ok(())
    }

    // ── Sync rules ────────────────────────────────────────────────────────────

    /// Create or re-enable a sync rule. Returns the rule id.
    pub async fn create_rule(
        &self,
        src_platform: &str,
        src_room:     &str,
        tgt_platform: &str,
        tgt_room:     &str,
        direction:    &str,
        sync_members: bool,
    ) -> Result<i64> {
        use fs_db::sea_orm::sea_query::OnConflict;
        sync_rule::Entity::insert(sync_rule::ActiveModel {
            source_platform: Set(src_platform.to_string()),
            source_room:     Set(src_room.to_string()),
            target_platform: Set(tgt_platform.to_string()),
            target_room:     Set(tgt_room.to_string()),
            direction:       Set(direction.to_string()),
            sync_members:    Set(sync_members as i64),
            enabled:         Set(1),
            created_at:      Set(Utc::now().to_rfc3339()),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([
                sync_rule::Column::SourcePlatform,
                sync_rule::Column::SourceRoom,
                sync_rule::Column::TargetPlatform,
                sync_rule::Column::TargetRoom,
            ])
            .update_columns([sync_rule::Column::Enabled, sync_rule::Column::Direction])
            .to_owned(),
        )
        .exec(&self.conn)
        .await?;
        let rule = sync_rule::Entity::find()
            .filter(sync_rule::Column::SourcePlatform.eq(src_platform))
            .filter(sync_rule::Column::SourceRoom.eq(src_room))
            .filter(sync_rule::Column::TargetPlatform.eq(tgt_platform))
            .filter(sync_rule::Column::TargetRoom.eq(tgt_room))
            .one(&self.conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("sync rule not found after upsert"))?;
        Ok(rule.id)
    }

    /// Disable (not delete) a sync rule. Returns true if a rule was found.
    pub async fn disable_rule(
        &self,
        src_platform: &str,
        src_room:     &str,
        tgt_platform: &str,
        tgt_room:     &str,
    ) -> Result<bool> {
        let res = sync_rule::Entity::update_many()
            .col_expr(sync_rule::Column::Enabled, Expr::value(0i64))
            .filter(sync_rule::Column::SourcePlatform.eq(src_platform))
            .filter(sync_rule::Column::SourceRoom.eq(src_room))
            .filter(sync_rule::Column::TargetPlatform.eq(tgt_platform))
            .filter(sync_rule::Column::TargetRoom.eq(tgt_room))
            .exec(&self.conn)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// Active sync rules where `platform`/`room` is source, or bidirectional target.
    pub async fn active_rules_for(&self, platform: &str, room: &str) -> Result<Vec<SyncRule>> {
        let rows = sync_rule::Entity::find()
            .filter(sync_rule::Column::Enabled.eq(1i64))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(sync_rule::Column::SourcePlatform.eq(platform))
                            .add(sync_rule::Column::SourceRoom.eq(room)),
                    )
                    .add(
                        Condition::all()
                            .add(sync_rule::Column::TargetPlatform.eq(platform))
                            .add(sync_rule::Column::TargetRoom.eq(room))
                            .add(sync_rule::Column::Direction.eq("both")),
                    ),
            )
            .all(&self.conn)
            .await?;
        Ok(rows.into_iter().map(SyncRule::from).collect())
    }

    /// All active sync rules (used by trigger handler on startup).
    pub async fn all_active_rules(&self) -> Result<Vec<SyncRule>> {
        let rows = sync_rule::Entity::find()
            .filter(sync_rule::Column::Enabled.eq(1i64))
            .all(&self.conn)
            .await?;
        Ok(rows.into_iter().map(SyncRule::from).collect())
    }

    /// Record a forwarded message for deduplication. Returns false if already forwarded.
    pub async fn record_forward(&self, rule_id: i64, direction: &str, msg_id_src: &str) -> Result<bool> {
        let exists = sync_message::Entity::find()
            .filter(sync_message::Column::RuleId.eq(rule_id))
            .filter(sync_message::Column::Direction.eq(direction))
            .filter(sync_message::Column::MsgIdSrc.eq(msg_id_src))
            .one(&self.conn)
            .await?
            .is_some();
        if exists { return Ok(false); }
        sync_message::ActiveModel {
            rule_id:      Set(rule_id),
            direction:    Set(direction.to_string()),
            msg_id_src:   Set(msg_id_src.to_string()),
            forwarded_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(true)
    }
}

// ── Filter types ──────────────────────────────────────────────────────────────

/// A resolved sync rule (domain type, not raw entity model).
#[derive(Debug, Clone)]
pub struct SyncRule {
    pub id:              i64,
    pub source_platform: String,
    pub source_room:     String,
    pub target_platform: String,
    pub target_room:     String,
    /// "both" | "to_target" | "to_source"
    pub direction:       String,
    pub sync_members:    bool,
}

impl From<sync_rule::Model> for SyncRule {
    fn from(m: sync_rule::Model) -> Self {
        Self {
            id:              m.id,
            source_platform: m.source_platform,
            source_room:     m.source_room,
            target_platform: m.target_platform,
            target_room:     m.target_room,
            direction:       m.direction,
            sync_members:    m.sync_members != 0,
        }
    }
}

/// Filter criteria for room queries — all fields optional, AND-combined.
#[derive(Debug, Default, Clone)]
pub struct GroupFilter {
    pub platform:      Option<String>,
    pub name_contains: Option<String>,
    pub min_members:   Option<i64>,
    pub max_members:   Option<i64>,
}
