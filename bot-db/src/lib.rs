#![deny(clippy::all, clippy::pedantic, warnings)]

// bot-db — BotDb object and all bot domain entities.
//
// The single database object for the bot manager. All bot sub-crates depend
// on this crate — never on sqlx or sea-orm directly.
//
// Uses fs-db DbEngine trait for all persistence. The underlying database
// backend (SQLite, Postgres, …) is injected at construction time.
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

use std::sync::Arc;

use anyhow::Result;
use fs_db::engine::DbEngine;

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

// ── Schema application ────────────────────────────────────────────────────────

/// Execute each `;`-delimited statement in `schema` via the engine.
///
/// `SQLite` pragmas and DDL statements (`CREATE TABLE IF NOT EXISTS …`) are
/// executed without parameters.
async fn apply_schema(engine: &dyn DbEngine, schema: &str) -> Result<()> {
    for raw in schema.split(';') {
        let stmt = raw.trim();
        if !stmt.is_empty() {
            engine
                .execute(stmt, vec![])
                .await
                .map_err(|e| anyhow::anyhow!("schema statement failed: {e}"))?;
        }
    }
    Ok(())
}

// ── BotDb ─────────────────────────────────────────────────────────────────────

/// The bot manager's database handle.
///
/// Wraps a [`DbEngine`] from `fs-db` and exposes typed repository
/// methods for every bot domain object. No raw SQL outside this crate.
#[derive(Clone)]
pub struct BotDb {
    engine: Arc<dyn DbEngine>,
}

impl BotDb {
    /// Create a `BotDb` backed by an existing engine and apply the schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be applied.
    pub async fn new(engine: Arc<dyn DbEngine>) -> Result<Self> {
        apply_schema(engine.as_ref(), SCHEMA).await?;
        Ok(Self { engine })
    }

    /// Open (or create) a `SQLite` database at `path` and apply the schema.
    ///
    /// This is a convenience constructor for the common single-node case.
    /// It requires the `sqlite` feature.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or the schema cannot be applied.
    #[cfg(feature = "sqlite")]
    pub async fn open(path: &str) -> Result<Self> {
        use fs_db::engine::DbConfig;
        use fs_db_engine_sqlite::SqliteEngine;
        let config = DbConfig::sqlite(path);
        let engine = SqliteEngine::open(config).await?;
        Self::new(Arc::new(engine)).await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn open_in_memory_applies_schema() {
        let db = BotDb::open(":memory:").await.expect("open failed");
        // Smoke-test: the bot_meta table should be queryable.
        db.set_meta("test-key", "test-value")
            .await
            .expect("set_meta failed");
        let val = db.get_meta("test-key").await.expect("get_meta failed");
        assert_eq!(val.as_deref(), Some("test-value"));
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn recent_audit_empty_initially() {
        let db = BotDb::open(":memory:").await.expect("open failed");
        let entries = db.recent_audit(10).await.expect("query failed");
        assert!(entries.is_empty());
    }
}
