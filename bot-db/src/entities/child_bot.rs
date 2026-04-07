// Child bot entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub bot_type: String,
    pub data_dir: String,
    pub status: String,
    pub pid: Option<i64>,
    pub created_at: String,
    pub started_at: Option<String>,
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
            name: row.get_string("name")?,
            bot_type: row.get_string("bot_type")?,
            data_dir: row.get_string("data_dir")?,
            status: row.get_string("status")?,
            pid: row.get_opt_i64("pid")?,
            created_at: row.get_string("created_at")?,
            started_at: row.get_opt_string("started_at")?,
        })
    }
}
