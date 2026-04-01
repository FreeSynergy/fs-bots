// audit.rs — Audit log methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, Order, QueryOrder, QuerySelect,
};

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
        audit_log::ActiveModel {
            actor_type: Set(entry.actor_type.to_string()),
            actor_id: Set(entry.actor_id.to_string()),
            platform: Set(entry.platform.map(str::to_string)),
            room_id: Set(entry.room_id.map(str::to_string)),
            action: Set(entry.action.to_string()),
            target: Set(entry.target.map(str::to_string)),
            result: Set(entry.result.to_string()),
            detail: Set(entry.detail.map(str::to_string)),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(())
    }

    /// Return the most recent `limit` audit log entries, newest first.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn recent_audit(&self, limit: u64) -> Result<Vec<audit_log::Model>> {
        Ok(audit_log::Entity::find()
            .order_by(audit_log::Column::Id, Order::Desc)
            .limit(limit)
            .all(&self.conn)
            .await?)
    }
}
