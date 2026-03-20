// bot-gatekeeper — N7: Join-request queue with IAM-based access control.
//
// Commands: /verify <user_id>, /approve <id>, /deny <id>
// Trigger:  GatekeeperHandler listens on "chat.join_request" and queues requests.

use fsn_bot::{CommandRegistry, TriggerHandler};
use sqlx::SqlitePool;

mod commands;
mod trigger;

/// Register all gatekeeper commands and return the trigger handler.
pub fn register(
    registry: &mut CommandRegistry,
    pool: SqlitePool,
) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, pool.clone());
    vec![Box::new(trigger::GatekeeperHandler::new(pool))]
}
