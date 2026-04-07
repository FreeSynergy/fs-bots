// Room collection entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
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
            name: row.get_string("name")?,
            description: row.get_opt_string("description")?,
            created_at: row.get_string("created_at")?,
        })
    }
}
