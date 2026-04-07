// Subscription entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub platform: String,
    pub room_id: String,
    pub topic: String,
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
            platform: row.get_string("platform")?,
            room_id: row.get_string("room_id")?,
            topic: row.get_string("topic")?,
            created_at: row.get_string("created_at")?,
        })
    }
}
