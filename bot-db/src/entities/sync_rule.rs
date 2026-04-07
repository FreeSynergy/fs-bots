// Sync rule entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub source_platform: String,
    pub source_room: String,
    pub target_platform: String,
    pub target_room: String,
    pub direction: String,
    pub sync_members: i64,
    pub enabled: i64,
    pub created_at: String,
}

impl Model {
    /// Build from a database row.
    ///
    /// # Errors
    ///
    /// Returns an error if a required column is missing or has the wrong type.
    pub fn from_row(row: &DbRow) -> Result<Self> {
        Ok(Self {
            id: row.get_i64("id")?,
            source_platform: row.get_string("source_platform")?,
            source_room: row.get_string("source_room")?,
            target_platform: row.get_string("target_platform")?,
            target_room: row.get_string("target_room")?,
            direction: row.get_string("direction")?,
            sync_members: row.get_i64("sync_members")?,
            enabled: row.get_i64("enabled")?,
            created_at: row.get_string("created_at")?,
        })
    }
}
