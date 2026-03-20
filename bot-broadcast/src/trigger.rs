// BroadcastHandler — forwards Bus events to subscribed rooms.

use async_trait::async_trait;
use fsn_bot::trigger::{TriggerAction, TriggerEvent, TriggerHandler};
use sqlx::{Row, SqlitePool};
use tracing::warn;

/// Listens on all Bus topics and sends events to rooms that have subscribed.
pub struct BroadcastHandler {
    pool: SqlitePool,
}

impl BroadcastHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TriggerHandler for BroadcastHandler {
    fn topics(&self) -> &[&str] {
        &["**"]
    }

    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction> {
        // Find all rooms subscribed to this exact topic
        let rows = sqlx::query(
            "SELECT platform, room_id FROM subscriptions WHERE topic = ?",
        )
        .bind(&event.topic)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_else(|e| {
            warn!("BroadcastHandler DB error: {e}");
            vec![]
        });

        let payload_str = match &event.payload {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        rows.into_iter()
            .map(|r| TriggerAction::SendToRoom {
                platform: r.get(0),
                room_id:  r.get(1),
                text:     format!("[{}] {}", event.topic, payload_str),
            })
            .collect()
    }
}
