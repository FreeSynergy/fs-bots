// Known room entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub platform: String,
    pub room_id: String,
    pub room_name: Option<String>,
    pub member_count: Option<i64>,
    pub last_seen: String,
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
            room_name: row.get_opt_string("room_name")?,
            member_count: row.get_opt_i64("member_count")?,
            last_seen: row.get_string("last_seen")?,
        })
    }
}
