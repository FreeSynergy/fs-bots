// Audit log facade — async, backed by BotDb.

use crate::db::BotDb;
use bot_db::AuditEntry;
use std::sync::Arc;

/// Shared audit log (`BotDb` is already clone-able via sqlx pool).
#[derive(Clone)]
pub struct AuditLog {
    db: Arc<BotDb>,
}

impl AuditLog {
    #[must_use]
    pub fn new(db: Arc<BotDb>) -> Self {
        Self { db }
    }

    pub async fn user_action(
        &self,
        user_id: &str,
        platform: &str,
        room_id: &str,
        action: &str,
        target: Option<&str>,
        result: &str,
        detail: Option<&str>,
    ) {
        if let Err(e) = self
            .db
            .audit(AuditEntry {
                actor_type: "user",
                actor_id: user_id,
                platform: Some(platform),
                room_id: Some(room_id),
                action,
                target,
                result,
                detail,
            })
            .await
        {
            tracing::warn!("audit write failed: {}", e);
        }
    }

    pub async fn system_action(
        &self,
        action: &str,
        platform: Option<&str>,
        room_id: Option<&str>,
        result: &str,
        detail: Option<&str>,
    ) {
        if let Err(e) = self
            .db
            .audit(AuditEntry {
                actor_type: "system",
                actor_id: "system",
                platform,
                room_id,
                action,
                target: None,
                result,
                detail,
            })
            .await
        {
            tracing::warn!("audit write failed: {}", e);
        }
    }
}
