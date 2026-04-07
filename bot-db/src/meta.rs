// meta.rs — Bot metadata key/value store methods for BotDb.

use anyhow::Result;
use serde_json::Value;

use crate::{entities::bot_meta, BotDb};

impl BotDb {
    /// Read a metadata key, returning `None` if not set.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM bot_meta WHERE key = ?",
                vec![Value::String(key.to_string())],
            )
            .await
            .map_err(|e| anyhow::anyhow!("get_meta query failed: {e}"))?;
        Ok(rows
            .rows
            .first()
            .and_then(|r| bot_meta::Model::from_row(r).ok())
            .map(|m| m.value))
    }

    /// Write (upsert) a metadata key/value pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.engine
            .execute(
                "INSERT INTO bot_meta (key, value) VALUES (?, ?) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                vec![
                    Value::String(key.to_string()),
                    Value::String(value.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("set_meta write failed: {e}"))?;
        Ok(())
    }
}
