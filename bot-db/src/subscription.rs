// subscription.rs — Room subscription methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::{entities::subscription, BotDb};

impl BotDb {
    /// Subscribe `platform`/`room_id` to `topic` (idempotent).
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn subscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        self.engine
            .execute(
                "INSERT OR IGNORE INTO subscriptions (platform, room_id, topic, created_at) \
                 VALUES (?, ?, ?, ?)",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                    Value::String(topic.to_string()),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("subscribe failed: {e}"))?;
        Ok(())
    }

    /// Remove the subscription of `platform`/`room_id` from `topic`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn unsubscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        self.engine
            .execute(
                "DELETE FROM subscriptions WHERE platform = ? AND room_id = ? AND topic = ?",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                    Value::String(topic.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("unsubscribe failed: {e}"))?;
        Ok(())
    }

    /// Return all topic names subscribed to by `platform`/`room_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn subscriptions_for_room(
        &self,
        platform: &str,
        room_id: &str,
    ) -> Result<Vec<String>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM subscriptions WHERE platform = ? AND room_id = ?",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("subscriptions_for_room query failed: {e}"))?;
        rows.rows
            .iter()
            .map(|r| subscription::Model::from_row(r).map(|m| m.topic))
            .collect()
    }

    /// All (platform, `room_id`) pairs subscribed to the given topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn subscriptions_for_room_by_topic(
        &self,
        topic: &str,
    ) -> Result<Vec<(String, String)>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM subscriptions WHERE topic = ?",
                vec![Value::String(topic.to_string())],
            )
            .await
            .map_err(|e| anyhow::anyhow!("subscriptions_for_room_by_topic query failed: {e}"))?;
        rows.rows
            .iter()
            .map(|r| subscription::Model::from_row(r).map(|m| (m.platform, m.room_id)))
            .collect()
    }
}
