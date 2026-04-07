// Sync message entity.

use anyhow::Result;
use fs_db::{engine::DbRow, record::DbRowExt};

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub id: i64,
    pub rule_id: i64,
    pub direction: String,
    pub msg_id_src: String,
    pub forwarded_at: String,
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
            rule_id: row.get_i64("rule_id")?,
            direction: row.get_string("direction")?,
            msg_id_src: row.get_string("msg_id_src")?,
            forwarded_at: row.get_string("forwarded_at")?,
        })
    }
}
