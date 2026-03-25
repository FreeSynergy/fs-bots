// bot-broadcast — N6: Subscribe rooms to Bus topics and broadcast events.
//
// Commands: /subscribe <topic>, /unsubscribe <topic>, /subscriptions
// Trigger:  BroadcastHandler listens on "**" and forwards events to subscribed rooms.

use bot_db::BotDb;
use fs_bot::{CommandRegistry, TriggerHandler};
use std::sync::Arc;

mod commands;
mod trigger;

/// Register all broadcast commands and return the trigger handler.
pub fn register(registry: &mut CommandRegistry, db: Arc<BotDb>) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, db.clone());
    vec![Box::new(trigger::BroadcastHandler::new(db))]
}
