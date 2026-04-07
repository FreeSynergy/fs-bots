// room.rs — Known room tracking methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

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
        self.engine
            .execute(
                "INSERT INTO known_rooms (platform, room_id, room_name, member_count, last_seen) \
                 VALUES (?, ?, ?, ?, ?) \
                 ON CONFLICT(platform, room_id) DO UPDATE SET \
                   room_name    = excluded.room_name, \
                   member_count = excluded.member_count, \
                   last_seen    = excluded.last_seen",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                    room_name.map_or(Value::Null, |v| Value::String(v.to_string())),
                    member_count.map_or(Value::Null, |v| Value::Number(v.into())),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("upsert_room failed: {e}"))?;
        Ok(())
    }

    /// Query known rooms, applying the given filter criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn filter_rooms(&self, filter: &GroupFilter) -> Result<Vec<known_room::Model>> {
        let mut sql = String::from("SELECT * FROM known_rooms WHERE 1=1");
        let mut params: Vec<Value> = Vec::new();

        if let Some(ref platform) = filter.platform {
            sql.push_str(" AND platform = ?");
            params.push(Value::String(platform.clone()));
        }
        if let Some(ref name) = filter.name_contains {
            sql.push_str(" AND room_name LIKE ?");
            params.push(Value::String(format!("%{name}%")));
        }
        if let Some(min) = filter.min_members {
            sql.push_str(" AND member_count >= ?");
            params.push(Value::Number(min.into()));
        }
        if let Some(max) = filter.max_members {
            sql.push_str(" AND member_count <= ?");
            params.push(Value::Number(max.into()));
        }
        sql.push_str(" ORDER BY room_name ASC");

        let rows = self
            .engine
            .execute(&sql, params)
            .await
            .map_err(|e| anyhow::anyhow!("filter_rooms query failed: {e}"))?;
        rows.rows.iter().map(known_room::Model::from_row).collect()
    }
}
