// BroadcastHandler — forwards Bus events to subscribed rooms.

use std::sync::Arc;
use async_trait::async_trait;
use bot_db::BotDb;
use fs_bot::trigger::{TriggerAction, TriggerEvent, TriggerHandler};
use tracing::warn;

/// Listens on all Bus topics and sends events to rooms that have subscribed.
pub struct BroadcastHandler {
    db: Arc<BotDb>,
}

impl BroadcastHandler {
    pub fn new(db: Arc<BotDb>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TriggerHandler for BroadcastHandler {
    fn topics(&self) -> &[&str] {
        &["**"]
    }

    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction> {
        let subscriptions = match self.db.subscriptions_for_room_by_topic(&event.topic).await {
            Ok(subs) => subs,
            Err(e) => {
                warn!("BroadcastHandler DB error: {e}");
                return vec![];
            }
        };

        let payload_str = match &event.payload {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        subscriptions
            .into_iter()
            .map(|(platform, room_id)| TriggerAction::SendToRoom {
                platform,
                room_id,
                text: format!("[{}] {}", event.topic, payload_str),
            })
            .collect()
    }
}
