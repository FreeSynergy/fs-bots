// Audit log entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub actor_type: String,
    pub actor_id: String,
    pub platform: Option<String>,
    pub room_id: Option<String>,
    pub action: String,
    pub target: Option<String>,
    pub result: String,
    pub detail: Option<String>,
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
            actor_type: row.get_string("actor_type")?,
            actor_id: row.get_string("actor_id")?,
            platform: row.get_opt_string("platform")?,
            room_id: row.get_opt_string("room_id")?,
            action: row.get_string("action")?,
            target: row.get_opt_string("target")?,
            result: row.get_string("result")?,
            detail: row.get_opt_string("detail")?,
            created_at: row.get_string("created_at")?,
        })
    }
}
