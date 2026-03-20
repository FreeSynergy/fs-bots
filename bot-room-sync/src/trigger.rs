// Room-sync trigger handler — forwards "chat.message" events to linked rooms.
//
// Cross-platform forwarding is handled by publishing a new "chat.message" Bus
// event addressed to the target room. The runtime routes it via the correct
// adapter. Same-platform rooms are forwarded directly.

use async_trait::async_trait;
use fsn_bot::{TriggerAction, TriggerEvent, TriggerHandler};
use serde_json::Value;
use sqlx::SqlitePool;
use tracing::warn;
use crate::SyncDb;

pub struct SyncHandler {
    db: SyncDb,
}

impl SyncHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { db: SyncDb::new(pool) }
    }
}

#[async_trait]
impl TriggerHandler for SyncHandler {
    fn topics(&self) -> &[&str] {
        &["chat.message"]
    }

    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction> {
        // Migrate schema on first event (bot may not have run /sync-start yet).
        if let Err(e) = self.db.migrate().await {
            warn!("sync: DB migration failed: {e}");
            return vec![];
        }

        let src_platform = match event.payload.get("platform").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None    => return vec![],
        };
        let src_room = match event.payload.get("room_id").and_then(Value::as_str) {
            Some(r) => r.to_string(),
            None    => return vec![],
        };
        let text = match event.payload.get("text").and_then(Value::as_str) {
            Some(t) => t.to_string(),
            None    => return vec![],
        };
        let sender = event.payload.get("sender").and_then(Value::as_str).unwrap_or("unknown");
        let msg_id = event.payload.get("message_id").and_then(Value::as_str).unwrap_or("");

        let rules = match self.db.active_rules_for(&src_platform, &src_room).await {
            Ok(r) => r,
            Err(e) => {
                warn!("sync: rule query failed: {e}");
                return vec![];
            }
        };

        let mut actions = Vec::new();

        for rule in &rules {
            // Determine forward direction
            let (forward_to_platform, forward_to_room): (String, String) =
                if rule.source_platform == src_platform
                    && rule.source_room == src_room
                    && (rule.direction == "both" || rule.direction == "to_target")
                {
                    (rule.target_platform.clone(), rule.target_room.clone())
                } else if rule.target_platform == src_platform
                    && rule.target_room == src_room
                    && rule.direction == "both"
                {
                    (rule.source_platform.clone(), rule.source_room.clone())
                } else {
                    continue;
                };

            // Deduplicate — skip if already forwarded
            if !msg_id.is_empty() {
                let direction_key = format!("{src_platform}→{forward_to_platform}");
                match self.db.record_forward(rule.id, &direction_key, msg_id).await {
                    Ok(false) => continue, // already forwarded
                    Err(e)    => { warn!("sync: dedup record failed: {e}"); }
                    Ok(true)  => {}
                }
            }

            let forwarded_text = format!("[{src_platform}/{src_room}] <{sender}> {text}");
            actions.push(TriggerAction::SendToRoom {
                platform: forward_to_platform.to_string(),
                room_id:  forward_to_room.to_string(),
                text:     forwarded_text,
            });
        }

        actions
    }
}
