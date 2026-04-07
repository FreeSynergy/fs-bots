// Join request entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub platform: String,
    pub room_id: String,
    pub user_id: String,
    pub status: String,
    pub iam_result: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
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
            user_id: row.get_string("user_id")?,
            status: row.get_string("status")?,
            iam_result: row.get_opt_string("iam_result")?,
            created_at: row.get_string("created_at")?,
            resolved_at: row.get_opt_string("resolved_at")?,
        })
    }
}
