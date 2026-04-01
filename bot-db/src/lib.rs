#![deny(clippy::all, clippy::pedantic, warnings)]

// bot-db — BotDb object and all bot domain entities.
//
// The single database object for the bot manager. All bot sub-crates depend
// on this crate — never on sqlx or sea-orm directly.
//
// Uses fs-db (SeaORM) for all persistence. The underlying database backend
// (SQLite, Postgres, …) is configured in fs-db — nothing here depends on it.
// To switch databases, only fs-db changes.
//
// Methods are split by domain concern into sub-modules:
//   audit        — AuditEntry + audit() / recent_audit()
//   poll         — get_offset() / set_offset()
//   room         — GroupFilter + upsert_room() / filter_rooms()
//   subscription — subscribe() / unsubscribe() / subscriptions_for_room*()
//   collection   — room collections (create / delete / list / members)
//   join         — join requests (add / get / list / resolve)
//   child        — child bot management (add / list / set_status)
//   meta         — key/value bot metadata (get / set)
//   sync         — SyncRule + create_rule() / disable_rule() / record_forward()

use anyhow::Result;
use fs_db::sea_orm::{ConnectionTrait, DatabaseConnection};

pub mod entities;

mod audit;
mod child;
mod collection;
mod join;
mod meta;
mod poll;
mod room;
mod subscription;
mod sync;

// Re-export public domain types from sub-modules.
pub use audit::AuditEntry;
pub use room::GroupFilter;
pub use sync::SyncRule;

const SCHEMA: &str = include_str!("../migrations/schema.sql");

// ── BotDb ─────────────────────────────────────────────────────────────────────

/// The bot manager's database handle.
///
/// Wraps a [`DatabaseConnection`] from `fs-db` and exposes typed repository
/// methods for every bot domain object. No raw SQL outside this crate.
#[derive(Clone)]
pub struct BotDb {
    conn: DatabaseConnection,
}

impl BotDb {
    /// Open (or create) the database at `path` and apply the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or the schema cannot be applied.
    pub async fn open(path: &str) -> Result<Self> {
        use fs_db::sea_orm::Database;
        let url = format!("sqlite://{path}?mode=rwc");
        let conn = Database::connect(&url).await?;
        conn.execute_unprepared(SCHEMA).await?;
        Ok(Self { conn })
    }
}
