// sync.rs — Sync rule and message deduplication methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::{entities::sync_rule, BotDb};

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
        self.engine
            .execute(
                "INSERT INTO sync_rules \
                 (source_platform, source_room, target_platform, target_room, \
                  direction, sync_members, enabled, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, 1, ?) \
                 ON CONFLICT(source_platform, source_room, target_platform, target_room) \
                 DO UPDATE SET enabled = 1, direction = excluded.direction",
                vec![
                    Value::String(src_platform.to_string()),
                    Value::String(src_room.to_string()),
                    Value::String(tgt_platform.to_string()),
                    Value::String(tgt_room.to_string()),
                    Value::String(direction.to_string()),
                    Value::Number(i64::from(sync_members).into()),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("create_rule upsert failed: {e}"))?;

        let rows = self
            .engine
            .execute(
                "SELECT id FROM sync_rules \
                 WHERE source_platform = ? AND source_room = ? \
                   AND target_platform = ? AND target_room = ?",
                vec![
                    Value::String(src_platform.to_string()),
                    Value::String(src_room.to_string()),
                    Value::String(tgt_platform.to_string()),
                    Value::String(tgt_room.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("create_rule fetch id failed: {e}"))?;
        rows.rows
            .first()
            .and_then(|r| r.values.first())
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("sync rule not found after upsert"))
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
        let result = self
            .engine
            .execute(
                "UPDATE sync_rules SET enabled = 0 \
                 WHERE source_platform = ? AND source_room = ? \
                   AND target_platform = ? AND target_room = ?",
                vec![
                    Value::String(src_platform.to_string()),
                    Value::String(src_room.to_string()),
                    Value::String(tgt_platform.to_string()),
                    Value::String(tgt_room.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("disable_rule failed: {e}"))?;
        Ok(result.rows_affected > 0)
    }

    /// Active sync rules where `platform`/`room` is source, or bidirectional target.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn active_rules_for(&self, platform: &str, room: &str) -> Result<Vec<SyncRule>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM sync_rules \
                 WHERE enabled = 1 AND ( \
                   (source_platform = ? AND source_room = ?) OR \
                   (target_platform = ? AND target_room = ? AND direction = 'both') \
                 )",
                vec![
                    Value::String(platform.to_string()),
                    Value::String(room.to_string()),
                    Value::String(platform.to_string()),
                    Value::String(room.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("active_rules_for query failed: {e}"))?;
        rows.rows
            .iter()
            .map(|r| sync_rule::Model::from_row(r).map(SyncRule::from))
            .collect()
    }

    /// All active sync rules (used by trigger handler on startup).
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn all_active_rules(&self) -> Result<Vec<SyncRule>> {
        let rows = self
            .engine
            .execute("SELECT * FROM sync_rules WHERE enabled = 1", vec![])
            .await
            .map_err(|e| anyhow::anyhow!("all_active_rules query failed: {e}"))?;
        rows.rows
            .iter()
            .map(|r| sync_rule::Model::from_row(r).map(SyncRule::from))
            .collect()
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
        let check = self
            .engine
            .execute(
                "SELECT id FROM sync_messages \
                 WHERE rule_id = ? AND direction = ? AND msg_id_src = ?",
                vec![
                    Value::Number(rule_id.into()),
                    Value::String(direction.to_string()),
                    Value::String(msg_id_src.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("record_forward check failed: {e}"))?;
        if !check.rows.is_empty() {
            return Ok(false);
        }
        self.engine
            .execute(
                "INSERT INTO sync_messages \
                 (rule_id, direction, msg_id_src, forwarded_at) VALUES (?, ?, ?, ?)",
                vec![
                    Value::Number(rule_id.into()),
                    Value::String(direction.to_string()),
                    Value::String(msg_id_src.to_string()),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("record_forward insert failed: {e}"))?;
        Ok(true)
    }
}
