// Bot-instance SQLite database — schema and access layer.
//
// Uses sqlx (same version as sea-orm in fsn-inventory) to avoid
// libsqlite3-sys version conflicts.
//
// Database file: <data_dir>/fsn-botmanager.db

use anyhow::Result;
use chrono::Utc;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use std::str::FromStr;

// ── Schema ────────────────────────────────────────────────────────────────────

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS bot_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    actor_type  TEXT    NOT NULL,
    actor_id    TEXT    NOT NULL,
    platform    TEXT,
    room_id     TEXT,
    action      TEXT    NOT NULL,
    target      TEXT,
    result      TEXT    NOT NULL,
    detail      TEXT,
    created_at  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    topic       TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    UNIQUE(platform, room_id, topic)
);

CREATE TABLE IF NOT EXISTS join_requests (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    user_id     TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'pending',
    iam_result  TEXT,
    created_at  TEXT    NOT NULL,
    resolved_at TEXT
);

CREATE TABLE IF NOT EXISTS known_rooms (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    room_name   TEXT,
    member_count INTEGER,
    last_seen   TEXT    NOT NULL,
    UNIQUE(platform, room_id)
);

CREATE TABLE IF NOT EXISTS poll_state (
    platform    TEXT    NOT NULL,
    room_id     TEXT    NOT NULL,
    last_offset INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (platform, room_id)
);

CREATE TABLE IF NOT EXISTS child_bots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    bot_type    TEXT    NOT NULL,
    data_dir    TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'stopped',
    pid         INTEGER,
    created_at  TEXT    NOT NULL,
    started_at  TEXT
);

CREATE TABLE IF NOT EXISTS room_collections (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    description TEXT,
    created_at  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS room_collection_members (
    collection_id INTEGER NOT NULL REFERENCES room_collections(id) ON DELETE CASCADE,
    platform      TEXT    NOT NULL,
    room_id       TEXT    NOT NULL,
    PRIMARY KEY (collection_id, platform, room_id)
);
"#;

// ── BotDb ─────────────────────────────────────────────────────────────────────

/// Async SQLite database handle for one bot instance.
#[derive(Clone)]
pub struct BotDb {
    pool: SqlitePool,
}

impl BotDb {
    /// Open (or create) the database and run migrations.
    pub async fn open(path: &str) -> Result<Self> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}?mode=rwc", path))?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);
        let pool = SqlitePool::connect_with(opts).await?;
        sqlx::query(SCHEMA).execute(&pool).await?;
        Ok(Self { pool })
    }

    // ── Audit ─────────────────────────────────────────────────────────────────

    pub async fn audit(
        &self,
        actor_type: &str,
        actor_id: &str,
        platform: Option<&str>,
        room_id: Option<&str>,
        action: &str,
        target: Option<&str>,
        result: &str,
        detail: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (actor_type, actor_id, platform, room_id, action, target, result, detail, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(actor_type).bind(actor_id).bind(platform).bind(room_id)
        .bind(action).bind(target).bind(result).bind(detail)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }

    // ── Poll state ────────────────────────────────────────────────────────────

    pub async fn get_offset(&self, platform: &str, room_id: &str) -> Result<u64> {
        let row = sqlx::query(
            "SELECT last_offset FROM poll_state WHERE platform = ? AND room_id = ?"
        )
        .bind(platform).bind(room_id)
        .fetch_optional(&self.pool).await?;
        Ok(row.map(|r| r.get::<i64, _>(0) as u64).unwrap_or(0))
    }

    pub async fn set_offset(&self, platform: &str, room_id: &str, offset: u64) -> Result<()> {
        sqlx::query(
            "INSERT INTO poll_state (platform, room_id, last_offset) VALUES (?, ?, ?)
             ON CONFLICT(platform, room_id) DO UPDATE SET last_offset = excluded.last_offset"
        )
        .bind(platform).bind(room_id).bind(offset as i64)
        .execute(&self.pool).await?;
        Ok(())
    }

    // ── Known rooms ───────────────────────────────────────────────────────────

    pub async fn upsert_room(&self, platform: &str, room_id: &str, room_name: Option<&str>, member_count: Option<i64>) -> Result<()> {
        sqlx::query(
            "INSERT INTO known_rooms (platform, room_id, room_name, member_count, last_seen) VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(platform, room_id) DO UPDATE SET room_name = excluded.room_name, member_count = excluded.member_count, last_seen = excluded.last_seen"
        )
        .bind(platform).bind(room_id).bind(room_name).bind(member_count)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }

    // ── Subscriptions ─────────────────────────────────────────────────────────

    pub async fn subscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO subscriptions (platform, room_id, topic, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(platform).bind(room_id).bind(topic).bind(Utc::now().to_rfc3339())
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn unsubscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        sqlx::query("DELETE FROM subscriptions WHERE platform = ? AND room_id = ? AND topic = ?")
            .bind(platform).bind(room_id).bind(topic)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn subscriptions_for_room(&self, platform: &str, room_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT topic FROM subscriptions WHERE platform = ? AND room_id = ?")
            .bind(platform).bind(room_id)
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.get::<String, _>(0)).collect())
    }

    // ── Room collections (N9) ─────────────────────────────────────────────────

    /// Create a new named collection.
    pub async fn create_collection(&self, name: &str, description: Option<&str>) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO room_collections (name, description, created_at) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(name).bind(description).bind(Utc::now().to_rfc3339())
        .fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Delete a collection (cascade removes members).
    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM room_collections WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    /// List all collections.
    pub async fn list_collections(&self) -> Result<Vec<RoomCollection>> {
        let rows = sqlx::query("SELECT id, name, description, created_at FROM room_collections ORDER BY name")
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| RoomCollection {
            id:          r.get(0),
            name:        r.get(1),
            description: r.get(2),
            created_at:  r.get(3),
        }).collect())
    }

    /// Add a room to a collection (idempotent).
    pub async fn add_to_collection(&self, collection_id: i64, platform: &str, room_id: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO room_collection_members (collection_id, platform, room_id) VALUES (?, ?, ?)",
        )
        .bind(collection_id).bind(platform).bind(room_id)
        .execute(&self.pool).await?;
        Ok(())
    }

    /// Remove a room from a collection.
    pub async fn remove_from_collection(&self, collection_id: i64, platform: &str, room_id: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM room_collection_members WHERE collection_id = ? AND platform = ? AND room_id = ?",
        )
        .bind(collection_id).bind(platform).bind(room_id)
        .execute(&self.pool).await?;
        Ok(())
    }

    /// List all rooms in a collection.
    pub async fn rooms_in_collection(&self, collection_id: i64) -> Result<Vec<RoomRef>> {
        let rows = sqlx::query(
            "SELECT m.platform, m.room_id, k.room_name, k.member_count
             FROM room_collection_members m
             LEFT JOIN known_rooms k ON k.platform = m.platform AND k.room_id = m.room_id
             WHERE m.collection_id = ?",
        )
        .bind(collection_id).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| RoomRef {
            platform:     r.get(0),
            room_id:      r.get(1),
            room_name:    r.get(2),
            member_count: r.get(3),
        }).collect())
    }

    /// All known rooms matching an optional filter.
    pub async fn filter_rooms(&self, filter: &GroupFilter) -> Result<Vec<KnownRoom>> {
        let mut sql = "SELECT platform, room_id, room_name, member_count, last_seen FROM known_rooms WHERE 1=1".to_string();
        let mut binds: Vec<String> = Vec::new();

        if let Some(ref platform) = filter.platform {
            sql.push_str(" AND platform = ?");
            binds.push(platform.clone());
        }
        if let Some(ref name) = filter.name_contains {
            sql.push_str(" AND room_name LIKE ?");
            binds.push(format!("%{name}%"));
        }
        if let Some(min) = filter.min_members {
            sql.push_str(&format!(" AND member_count >= {min}"));
        }
        if let Some(max) = filter.max_members {
            sql.push_str(&format!(" AND member_count <= {max}"));
        }
        sql.push_str(" ORDER BY room_name");

        let mut q = sqlx::query(&sql);
        for b in &binds { q = q.bind(b.clone()); }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| KnownRoom {
            platform:     r.get(0),
            room_id:      r.get(1),
            room_name:    r.get(2),
            member_count: r.get(3),
            last_seen:    r.get(4),
        }).collect())
    }

    // ── Pool access (for module crates) ───────────────────────────────────────

    /// Return a clone of the underlying SQLite connection pool.
    ///
    /// `SqlitePool` is internally reference-counted — cloning is cheap.
    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    // ── Join requests (Gatekeeper) ────────────────────────────────────────────

    /// Insert a new join request and return its row id.
    pub async fn add_join_request(
        &self,
        platform: &str,
        room_id: &str,
        user_id: &str,
    ) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO join_requests (platform, room_id, user_id, status, created_at)
             VALUES (?, ?, ?, 'pending', ?) RETURNING id",
        )
        .bind(platform)
        .bind(room_id)
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Fetch a join request by id.
    pub async fn get_join_request(&self, id: i64) -> Result<Option<JoinRequest>> {
        let row = sqlx::query(
            "SELECT id, platform, room_id, user_id, status, iam_result, created_at, resolved_at
             FROM join_requests WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| JoinRequest {
            id:          r.get(0),
            platform:    r.get(1),
            room_id:     r.get(2),
            user_id:     r.get(3),
            status:      r.get(4),
            iam_result:  r.get(5),
            created_at:  r.get(6),
            resolved_at: r.get(7),
        }))
    }

    /// List all pending join requests for a room.
    pub async fn list_pending_join_requests(
        &self,
        platform: &str,
        room_id: &str,
    ) -> Result<Vec<JoinRequest>> {
        let rows = sqlx::query(
            "SELECT id, platform, room_id, user_id, status, iam_result, created_at, resolved_at
             FROM join_requests WHERE platform = ? AND room_id = ? AND status = 'pending'
             ORDER BY created_at",
        )
        .bind(platform)
        .bind(room_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| JoinRequest {
                id:          r.get(0),
                platform:    r.get(1),
                room_id:     r.get(2),
                user_id:     r.get(3),
                status:      r.get(4),
                iam_result:  r.get(5),
                created_at:  r.get(6),
                resolved_at: r.get(7),
            })
            .collect())
    }

    /// Resolve a join request (approve or deny).
    pub async fn resolve_join_request(
        &self,
        id: i64,
        status: &str,
        iam_result: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE join_requests SET status = ?, iam_result = ?, resolved_at = ?
             WHERE id = ?",
        )
        .bind(status)
        .bind(iam_result)
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Audit query ───────────────────────────────────────────────────────────

    /// Return the most recent `limit` audit log entries (newest first).
    pub async fn recent_audit(&self, limit: i64) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query(
            "SELECT id, actor_type, actor_id, platform, action, result, created_at
             FROM audit_log ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| AuditEntry {
                id:         r.get(0),
                actor_type: r.get(1),
                actor_id:   r.get(2),
                platform:   r.get(3),
                action:     r.get(4),
                result:     r.get(5),
                created_at: r.get(6),
            })
            .collect())
    }

    // ── Child bots (Control) ──────────────────────────────────────────────────

    /// Register a child bot entry.
    pub async fn add_child_bot(&self, name: &str, bot_type: &str, data_dir: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO child_bots (name, bot_type, data_dir, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(name)
        .bind(bot_type)
        .bind(data_dir)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// List all child bots.
    pub async fn list_child_bots(&self) -> Result<Vec<ChildBot>> {
        let rows = sqlx::query(
            "SELECT id, name, bot_type, data_dir, status, pid, created_at
             FROM child_bots ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| ChildBot {
                id:         r.get(0),
                name:       r.get(1),
                bot_type:   r.get(2),
                data_dir:   r.get(3),
                status:     r.get(4),
                pid:        r.get(5),
                created_at: r.get(6),
            })
            .collect())
    }

    /// Update child bot status (e.g. "running" / "stopped").
    pub async fn set_child_bot_status(
        &self,
        name: &str,
        status: &str,
        pid: Option<i64>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE child_bots SET status = ?, pid = ? WHERE name = ?",
        )
        .bind(status)
        .bind(pid)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// ── Result types ──────────────────────────────────────────────────────────────

/// A join request record.
#[derive(Debug)]
pub struct JoinRequest {
    pub id: i64,
    pub platform: String,
    pub room_id: String,
    pub user_id: String,
    pub status: String,
    pub iam_result: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

/// An audit log entry.
#[derive(Debug)]
pub struct AuditEntry {
    pub id: i64,
    pub actor_type: String,
    pub actor_id: String,
    pub platform: Option<String>,
    pub action: String,
    pub result: String,
    pub created_at: String,
}

/// A room collection (manual group of rooms).
#[derive(Debug, Clone)]
pub struct RoomCollection {
    pub id:          i64,
    pub name:        String,
    pub description: Option<String>,
    pub created_at:  String,
}

/// A lightweight room reference (platform + room_id).
#[derive(Debug, Clone)]
pub struct RoomRef {
    pub platform:     String,
    pub room_id:      String,
    pub room_name:    Option<String>,
    pub member_count: Option<i64>,
}

/// A full known-room record (from the `known_rooms` table).
#[derive(Debug, Clone)]
pub struct KnownRoom {
    pub platform:     String,
    pub room_id:      String,
    pub room_name:    Option<String>,
    pub member_count: Option<i64>,
    pub last_seen:    String,
}

/// Filter criteria for room queries (all fields optional, AND-combined).
#[derive(Debug, Default, Clone)]
pub struct GroupFilter {
    /// Filter by messenger platform name.
    pub platform:       Option<String>,
    /// Substring match on room name (case-insensitive via LIKE).
    pub name_contains:  Option<String>,
    /// Minimum member count (inclusive).
    pub min_members:    Option<i64>,
    /// Maximum member count (inclusive).
    pub max_members:    Option<i64>,
}

/// A registered child bot.
#[derive(Debug)]
pub struct ChildBot {
    pub id: i64,
    pub name: String,
    pub bot_type: String,
    pub data_dir: String,
    pub status: String,
    pub pid: Option<i64>,
    pub created_at: String,
}
