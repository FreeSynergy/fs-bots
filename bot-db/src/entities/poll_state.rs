// Poll state entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub platform: String,
    pub room_id: String,
    pub last_offset: i64,
}

impl Model {
    /// Build from a database row.
    ///
    /// # Errors
    ///
    /// Returns an error if a required column is missing or has the wrong type.
    pub fn from_row(row: &DbRow) -> Result<Self> {
        Ok(Self {
            platform: row.get_string("platform")?,
            room_id: row.get_string("room_id")?,
            last_offset: row.get_i64("last_offset")?,
        })
    }
}
