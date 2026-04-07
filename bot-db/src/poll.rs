// poll.rs — Poll state (message offset tracking) methods for BotDb.

use anyhow::Result;
use serde_json::Value;

use crate::{entities::poll_state, BotDb};

impl BotDb {
    /// Return the last poll offset for `platform`/`room_id`, or 0 if not set.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_offset(&self, platform: &str, room_id: &str) -> Result<u64> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM poll_state WHERE platform = ? AND room_id = ?",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("get_offset query failed: {e}"))?;
        Ok(rows
            .rows
            .first()
            .and_then(|r| poll_state::Model::from_row(r).ok())
            .map_or(0, |m| u64::try_from(m.last_offset).unwrap_or(0)))
    }

    /// Persist the poll offset for `platform`/`room_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn set_offset(&self, platform: &str, room_id: &str, offset: u64) -> Result<()> {
        self.engine
            .execute(
                "INSERT INTO poll_state (platform, room_id, last_offset) VALUES (?, ?, ?) \
                 ON CONFLICT(platform, room_id) DO UPDATE SET last_offset = excluded.last_offset",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                    Value::Number(i64::try_from(offset).unwrap_or(i64::MAX).into()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("set_offset write failed: {e}"))?;
        Ok(())
    }
}
