// Room collection member entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub collection_id: i64,
    pub platform: String,
    pub room_id: String,
}

impl Model {
    /// Build from a database row.
    ///
    /// # Errors
    ///
    /// Returns an error if a required column is missing or has the wrong type.
    pub fn from_row(row: &DbRow) -> Result<Self> {
        Ok(Self {
            collection_id: row.get_i64("collection_id")?,
            platform: row.get_string("platform")?,
            room_id: row.get_string("room_id")?,
        })
    }
}
