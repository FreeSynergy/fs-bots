// join.rs — Join request methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::{entities::join_request, BotDb};

impl BotDb {
    /// Queue a new join request. Returns the generated request id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn add_join_request(
        &self,
        platform: &str,
        room_id: &str,
        user_id: &str,
    ) -> Result<i64> {
        self.engine
            .execute(
                "INSERT INTO join_requests \
                 (platform, room_id, user_id, status, created_at) \
                 VALUES (?, ?, ?, 'pending', ?)",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                    Value::String(user_id.to_string()),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("add_join_request failed: {e}"))?;
        let id_rows = self
            .engine
            .execute("SELECT last_insert_rowid() AS id", vec![])
            .await
            .map_err(|e| anyhow::anyhow!("last_insert_rowid failed: {e}"))?;
        id_rows
            .rows
            .first()
            .and_then(|r| r.values.first())
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("add_join_request: no rowid returned"))
    }

    /// Fetch a single join request by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_join_request(&self, id: i64) -> Result<Option<join_request::Model>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM join_requests WHERE id = ?",
                vec![Value::Number(id.into())],
            )
            .await
            .map_err(|e| anyhow::anyhow!("get_join_request query failed: {e}"))?;
        rows.rows
            .first()
            .map(join_request::Model::from_row)
            .transpose()
    }

    /// List all pending join requests for `platform`/`room_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_pending_join_requests(
        &self,
        platform: &str,
        room_id: &str,
    ) -> Result<Vec<join_request::Model>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM join_requests \
                 WHERE platform = ? AND room_id = ? AND status = 'pending' \
                 ORDER BY created_at ASC",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("list_pending_join_requests query failed: {e}"))?;
        rows.rows
            .iter()
            .map(join_request::Model::from_row)
            .collect()
    }

    /// Update the status of a join request.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn resolve_join_request(
        &self,
        id: i64,
        status: &str,
        iam_result: Option<&str>,
    ) -> Result<()> {
        self.engine
            .execute(
                "UPDATE join_requests \
                 SET status = ?, iam_result = ?, resolved_at = ? \
                 WHERE id = ?",
                vec![
                    Value::String(status.to_string()),
                    iam_result.map_or(Value::Null, |v| Value::String(v.to_string())),
                    Value::String(Utc::now().to_rfc3339()),
                    Value::Number(id.into()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("resolve_join_request failed: {e}"))?;
        Ok(())
    }
}
