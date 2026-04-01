// sync.rs — Sync rule and message deduplication methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::sea_query::Expr;
use fs_db::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, QueryFilter,
};

use crate::{
    entities::{sync_message, sync_rule},
    BotDb,
};

/// A resolved sync rule (domain type, not raw entity model).
#[derive(Debug, Clone)]
pub struct SyncRule {
    pub id: i64,
    pub source_platform: String,
    pub source_room: String,
    pub target_platform: String,
    pub target_room: String,
    /// `"both"` | `"to_target"` | `"to_source"`
    pub direction: String,
    pub sync_members: bool,
}

impl From<sync_rule::Model> for SyncRule {
    fn from(m: sync_rule::Model) -> Self {
        Self {
            id: m.id,
            source_platform: m.source_platform,
            source_room: m.source_room,
            target_platform: m.target_platform,
            target_room: m.target_room,
            direction: m.direction,
            sync_members: m.sync_members != 0,
        }
    }
}

impl BotDb {
    /// Create or re-enable a sync rule. Returns the rule id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails or the rule cannot be found after upsert.
    pub async fn create_rule(
        &self,
        src_platform: &str,
        src_room: &str,
        tgt_platform: &str,
        tgt_room: &str,
        direction: &str,
        sync_members: bool,
    ) -> Result<i64> {
        use fs_db::sea_orm::sea_query::OnConflict;
        sync_rule::Entity::insert(sync_rule::ActiveModel {
            source_platform: Set(src_platform.to_string()),
            source_room: Set(src_room.to_string()),
            target_platform: Set(tgt_platform.to_string()),
            target_room: Set(tgt_room.to_string()),
            direction: Set(direction.to_string()),
            sync_members: Set(i64::from(sync_members)),
            enabled: Set(1),
            created_at: Set(Utc::now().to_rfc3339()),
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

    /// Disable (not delete) a sync rule. Returns `true` if a rule was found.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn disable_rule(
        &self,
        src_platform: &str,
        src_room: &str,
        tgt_platform: &str,
        tgt_room: &str,
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
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn all_active_rules(&self) -> Result<Vec<SyncRule>> {
        let rows = sync_rule::Entity::find()
            .filter(sync_rule::Column::Enabled.eq(1i64))
            .all(&self.conn)
            .await?;
        Ok(rows.into_iter().map(SyncRule::from).collect())
    }

    /// Record a forwarded message for deduplication. Returns `false` if already forwarded.
    ///
    /// # Errors
    ///
    /// Returns an error if the database read or write fails.
    pub async fn record_forward(
        &self,
        rule_id: i64,
        direction: &str,
        msg_id_src: &str,
    ) -> Result<bool> {
        let exists = sync_message::Entity::find()
            .filter(sync_message::Column::RuleId.eq(rule_id))
            .filter(sync_message::Column::Direction.eq(direction))
            .filter(sync_message::Column::MsgIdSrc.eq(msg_id_src))
            .one(&self.conn)
            .await?
            .is_some();
        if exists {
            return Ok(false);
        }
        sync_message::ActiveModel {
            rule_id: Set(rule_id),
            direction: Set(direction.to_string()),
            msg_id_src: Set(msg_id_src.to_string()),
            forwarded_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(true)
    }
}
