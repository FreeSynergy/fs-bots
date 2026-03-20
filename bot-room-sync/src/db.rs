// Room-sync SQLite helpers.

use anyhow::Result;
use chrono::Utc;
use sqlx::{SqlitePool, Row};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sync_rules (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_platform TEXT    NOT NULL,
    source_room     TEXT    NOT NULL,
    target_platform TEXT    NOT NULL,
    target_room     TEXT    NOT NULL,
    direction       TEXT    NOT NULL DEFAULT 'both',
    sync_members    INTEGER NOT NULL DEFAULT 0,
    enabled         INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT    NOT NULL,
    UNIQUE(source_platform, source_room, target_platform, target_room)
);

CREATE TABLE IF NOT EXISTS sync_messages (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id       INTEGER NOT NULL REFERENCES sync_rules(id) ON DELETE CASCADE,
    direction     TEXT    NOT NULL,
    msg_id_src    TEXT    NOT NULL,
    forwarded_at  TEXT    NOT NULL
);
"#;

// ── SyncDb ────────────────────────────────────────────────────────────────────

/// Thin wrapper around the shared pool for sync-related queries.
#[derive(Clone)]
pub struct SyncDb {
    pool: SqlitePool,
}

impl SyncDb {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Run the sync schema migration (idempotent).
    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(SCHEMA).execute(&self.pool).await?;
        Ok(())
    }

    /// Create or re-enable a sync rule.
    pub async fn create_rule(
        &self,
        src_platform: &str,
        src_room:     &str,
        tgt_platform: &str,
        tgt_room:     &str,
        direction:    &str,  // "both" | "to_target" | "to_source"
        sync_members: bool,
    ) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO sync_rules
                (source_platform, source_room, target_platform, target_room, direction, sync_members, enabled, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 1, ?)
             ON CONFLICT(source_platform, source_room, target_platform, target_room)
             DO UPDATE SET enabled = 1, direction = excluded.direction
             RETURNING id",
        )
        .bind(src_platform).bind(src_room)
        .bind(tgt_platform).bind(tgt_room)
        .bind(direction).bind(sync_members as i64)
        .bind(Utc::now().to_rfc3339())
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Disable (not delete) a sync rule.
    pub async fn disable_rule(
        &self,
        src_platform: &str,
        src_room:     &str,
        tgt_platform: &str,
        tgt_room:     &str,
    ) -> Result<bool> {
        let res = sqlx::query(
            "UPDATE sync_rules SET enabled = 0
             WHERE source_platform = ? AND source_room = ? AND target_platform = ? AND target_room = ?",
        )
        .bind(src_platform).bind(src_room).bind(tgt_platform).bind(tgt_room)
        .execute(&self.pool).await?;
        Ok(res.rows_affected() > 0)
    }

    /// List active sync rules for a source room.
    pub async fn active_rules_for(&self, platform: &str, room: &str) -> Result<Vec<SyncRule>> {
        let rows = sqlx::query(
            "SELECT id, source_platform, source_room, target_platform, target_room, direction, sync_members
             FROM sync_rules WHERE enabled = 1
             AND (source_platform = ? AND source_room = ?
                  OR (target_platform = ? AND target_room = ? AND direction = 'both'))",
        )
        .bind(platform).bind(room).bind(platform).bind(room)
        .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| SyncRule {
            id:              r.get(0),
            source_platform: r.get(1),
            source_room:     r.get(2),
            target_platform: r.get(3),
            target_room:     r.get(4),
            direction:       r.get(5),
            sync_members:    r.get::<i64, _>(6) != 0,
        }).collect())
    }

    /// Record a forwarded message (deduplication).
    pub async fn record_forward(&self, rule_id: i64, direction: &str, msg_id_src: &str) -> Result<bool> {
        let exists: bool = sqlx::query(
            "SELECT 1 FROM sync_messages WHERE rule_id = ? AND direction = ? AND msg_id_src = ?",
        )
        .bind(rule_id).bind(direction).bind(msg_id_src)
        .fetch_optional(&self.pool).await?.is_some();
        if exists { return Ok(false); }
        sqlx::query(
            "INSERT INTO sync_messages (rule_id, direction, msg_id_src, forwarded_at) VALUES (?, ?, ?, ?)",
        )
        .bind(rule_id).bind(direction).bind(msg_id_src).bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(true)
    }

    /// All active sync rules (for the trigger handler to load on startup).
    pub async fn all_active_rules(&self) -> Result<Vec<SyncRule>> {
        let rows = sqlx::query(
            "SELECT id, source_platform, source_room, target_platform, target_room, direction, sync_members
             FROM sync_rules WHERE enabled = 1",
        )
        .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| SyncRule {
            id:              r.get(0),
            source_platform: r.get(1),
            source_room:     r.get(2),
            target_platform: r.get(3),
            target_room:     r.get(4),
            direction:       r.get(5),
            sync_members:    r.get::<i64, _>(6) != 0,
        }).collect())
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SyncRule {
    pub id:              i64,
    pub source_platform: String,
    pub source_room:     String,
    pub target_platform: String,
    pub target_room:     String,
    /// "both" | "to_target" | "to_source"
    pub direction:       String,
    pub sync_members:    bool,
}
