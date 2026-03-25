// bot-gatekeeper — N7: Join-request queue with IAM-based access control.
//
// Commands: /verify <user_id>, /approve <id>, /deny <id>
// Trigger:  GatekeeperHandler listens on "chat.join_request" and queues requests.

use bot_db::BotDb;
use fs_bot::{CommandRegistry, TriggerHandler};
use std::sync::Arc;

mod commands;
mod trigger;

/// Register all gatekeeper commands and return the trigger handler.
pub fn register(registry: &mut CommandRegistry, db: Arc<BotDb>) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, db.clone());
    vec![Box::new(trigger::GatekeeperHandler::new(db))]
}
