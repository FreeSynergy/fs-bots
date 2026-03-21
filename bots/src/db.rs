// Re-export BotDb from the shared bot-db crate.
//
// All persistence goes through bot-db::BotDb. To switch the database backend,
// only fs-db changes — nothing here, and nothing in the sub-bots.

pub use bot_db::{BotDb, GroupFilter, SyncRule};
