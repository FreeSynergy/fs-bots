// bot-room-sync — N10: Synchronize messages and members across rooms / platforms.
//
// Schema tables: sync_rules, sync_messages (managed by bot-db).
//
// Commands:
//   /sync-start <target_platform> <target_room>  — create bidirectional sync rule
//   /sync-stop  <target_platform> <target_room>  — disable sync rule
//   /sync-status                                 — list active sync rules for this room
//
// Trigger: handles "chat.message" events and forwards them to linked rooms.

use std::sync::Arc;
use bot_db::BotDb;
use fs_bot::{CommandRegistry, TriggerHandler};

mod commands;
mod trigger;

/// Register all room-sync commands and return the trigger handler.
pub fn register(registry: &mut CommandRegistry, db: Arc<BotDb>) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, db.clone());
    vec![Box::new(trigger::SyncHandler::new(db))]
}
