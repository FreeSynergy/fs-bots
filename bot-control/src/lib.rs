#![deny(clippy::all, clippy::pedantic, warnings)]

// bot-control — N5: Manage child bot instances.
//
// Commands: /bots, /bot-create <name> <type>, /bot-status <name>, /bot-logs [n]
// Trigger:  ControlHandler listens on "bot.**" events.

use bot_db::BotDb;
use fs_bot::{CommandRegistry, TriggerHandler};
use std::sync::Arc;

mod commands;
mod trigger;

/// Register control commands and return the trigger handler.
pub fn register(registry: &mut CommandRegistry, db: Arc<BotDb>) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, db);
    vec![Box::new(trigger::ControlHandler)]
}
