// bot-broadcast — N6: Subscribe rooms to Bus topics and broadcast events.
//
// Commands: /subscribe <topic>, /unsubscribe <topic>, /subscriptions
// Trigger:  BroadcastHandler listens on "**" and forwards events to subscribed rooms.

use fsn_bot::{CommandRegistry, TriggerHandler};
use sqlx::SqlitePool;

mod commands;
mod trigger;

/// Register all broadcast commands and return the trigger handler.
///
/// The caller (runtime) registers the returned handlers into `TriggerEngine`.
pub fn register(
    registry: &mut CommandRegistry,
    pool: SqlitePool,
) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, pool.clone());
    vec![Box::new(trigger::BroadcastHandler::new(pool))]
}
