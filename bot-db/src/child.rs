// child.rs — Child bot management methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::{entities::child_bot, BotDb};

impl BotDb {
    /// Register a new child bot (idempotent by name).
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn add_child_bot(&self, name: &str, bot_type: &str, data_dir: &str) -> Result<()> {
        self.engine
            .execute(
                "INSERT OR IGNORE INTO child_bots \
                 (name, bot_type, data_dir, status, created_at) \
                 VALUES (?, ?, ?, 'stopped', ?)",
                vec![
                    Value::String(name.to_string()),
                    Value::String(bot_type.to_string()),
                    Value::String(data_dir.to_string()),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("add_child_bot failed: {e}"))?;
        Ok(())
    }

    /// List all registered child bots, ordered by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_child_bots(&self) -> Result<Vec<child_bot::Model>> {
        let rows = self
            .engine
            .execute("SELECT * FROM child_bots ORDER BY name ASC", vec![])
            .await
            .map_err(|e| anyhow::anyhow!("list_child_bots query failed: {e}"))?;
        rows.rows.iter().map(child_bot::Model::from_row).collect()
    }

    /// Update the runtime status and PID of a child bot.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot is not found or the database write fails.
    pub async fn set_child_bot_status(
        &self,
        name: &str,
        status: &str,
        pid: Option<i64>,
    ) -> Result<()> {
        self.engine
            .execute(
                "UPDATE child_bots SET status = ?, pid = ? WHERE name = ?",
                vec![
                    Value::String(status.to_string()),
                    pid.map_or(Value::Null, |v| Value::Number(v.into())),
                    Value::String(name.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("set_child_bot_status failed: {e}"))?;
        Ok(())
    }
}
