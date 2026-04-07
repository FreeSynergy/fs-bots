// audit.rs — Audit log methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::{entities::audit_log, BotDb};

/// All fields needed for one audit log entry.
#[derive(Debug, Clone)]
pub struct AuditEntry<'a> {
    pub actor_type: &'a str,
    pub actor_id: &'a str,
    pub platform: Option<&'a str>,
    pub room_id: Option<&'a str>,
    pub action: &'a str,
    pub target: Option<&'a str>,
    pub result: &'a str,
    pub detail: Option<&'a str>,
}

impl BotDb {
    /// Write one audit log entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn audit(&self, entry: AuditEntry<'_>) -> Result<()> {
        self.engine
            .execute(
                "INSERT INTO audit_log \
                 (actor_type, actor_id, platform, room_id, action, target, result, detail, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                vec![
                    Value::String(entry.actor_type.to_string()),
                    Value::String(entry.actor_id.to_string()),
                    entry.platform.map_or(Value::Null, |v| Value::String(v.to_string())),
                    entry.room_id.map_or(Value::Null, |v| Value::String(v.to_string())),
                    Value::String(entry.action.to_string()),
                    entry.target.map_or(Value::Null, |v| Value::String(v.to_string())),
                    Value::String(entry.result.to_string()),
                    entry.detail.map_or(Value::Null, |v| Value::String(v.to_string())),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("audit insert failed: {e}"))?;
        Ok(())
    }

    /// Return the most recent `limit` audit log entries, newest first.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn recent_audit(&self, limit: u64) -> Result<Vec<audit_log::Model>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM audit_log ORDER BY id DESC LIMIT ?",
                vec![Value::Number(limit.into())],
            )
            .await
            .map_err(|e| anyhow::anyhow!("recent_audit query failed: {e}"))?;
        rows.rows.iter().map(audit_log::Model::from_row).collect()
    }
}
