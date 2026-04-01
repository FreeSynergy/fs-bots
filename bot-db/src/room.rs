// room.rs — Known room tracking methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::{entities::known_room, BotDb};

/// Filter criteria for room queries — all fields optional, AND-combined.
#[derive(Debug, Default, Clone)]
pub struct GroupFilter {
    pub platform: Option<String>,
    pub name_contains: Option<String>,
    pub min_members: Option<i64>,
    pub max_members: Option<i64>,
}

impl BotDb {
    /// Insert or update a known room record.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn upsert_room(
        &self,
        platform: &str,
        room_id: &str,
        room_name: Option<&str>,
        member_count: Option<i64>,
    ) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        let model = known_room::ActiveModel {
            platform: Set(platform.to_string()),
            room_id: Set(room_id.to_string()),
            room_name: Set(room_name.map(str::to_string)),
            member_count: Set(member_count),
            last_seen: Set(Utc::now().to_rfc3339()),
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

    /// Query known rooms, applying the given filter criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
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
        Ok(query
            .order_by_asc(known_room::Column::RoomName)
            .all(&self.conn)
            .await?)
    }
}
