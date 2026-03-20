// bot-control — N5: Manage child bot instances.
//
// Commands: /bots, /bot-create <name> <type>, /bot-status <name>, /bot-logs [n]
// Trigger:  ControlHandler listens on "bot.**" events.

use fsn_bot::{CommandRegistry, TriggerHandler};
use sqlx::SqlitePool;

mod commands;
mod trigger;

/// Register control commands and return the trigger handler.
pub fn register(
    registry: &mut CommandRegistry,
    pool: SqlitePool,
) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, pool.clone());
    vec![Box::new(trigger::ControlHandler)]
}
