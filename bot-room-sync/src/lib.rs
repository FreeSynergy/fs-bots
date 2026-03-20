// bot-room-sync — N10: Synchronize messages and members across rooms / platforms.
//
// Schema (separate tables inside the shared bot DB pool):
//   sync_rules  — (id, source_platform, source_room, target_platform, target_room,
//                   direction [both|to_target|to_source], sync_members, enabled, created_at)
//   sync_messages — (id, rule_id, direction [fwd|bwd], msg_id_src, forwarded_at)
//
// Commands:
//   /sync-start <target_platform> <target_room>  — create bidirectional sync rule
//   /sync-stop  <target_platform> <target_room>  — disable sync rule
//   /sync-status                                 — list active sync rules for this room
//
// Trigger: handles "chat.message" events and forwards them to linked rooms.

use fsn_bot::{CommandRegistry, TriggerHandler};
use sqlx::SqlitePool;

mod commands;
mod db;
mod trigger;

pub use db::SyncDb;

/// Register all room-sync commands and return the trigger handler.
pub fn register(registry: &mut CommandRegistry, pool: SqlitePool) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry, pool.clone());
    vec![Box::new(trigger::SyncHandler::new(pool))]
}
