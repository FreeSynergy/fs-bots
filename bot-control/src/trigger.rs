// ControlHandler — listens on "bot.**" Bus events and logs them.

use async_trait::async_trait;
use fs_bot::trigger::{TriggerAction, TriggerEvent, TriggerHandler};
use tracing::info;

/// Listens on `bot.**` events.
///
/// In Phase N11, this will forward status requests to the `BotManager`.
pub struct ControlHandler;

#[async_trait]
impl TriggerHandler for ControlHandler {
    fn topics(&self) -> &[&str] {
        &["bot.**"]
    }

    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction> {
        info!(
            "ControlHandler: received bot event '{}': {}",
            event.topic, event.payload
        );
        // TODO Phase N11: route bot.status.request → BotManager via Bus
        vec![]
    }
}
