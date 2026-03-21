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
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    platform     TEXT    NOT NULL,
    room_id      TEXT    NOT NULL,
    room_name    TEXT,
    member_count INTEGER,
    last_seen    TEXT    NOT NULL,
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
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id      INTEGER NOT NULL REFERENCES sync_rules(id) ON DELETE CASCADE,
    direction    TEXT    NOT NULL,
    msg_id_src   TEXT    NOT NULL,
    forwarded_at TEXT    NOT NULL
);
