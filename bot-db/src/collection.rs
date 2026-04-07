// collection.rs — Room collection methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::{
    entities::{room_collection, room_collection_member},
    BotDb,
};

impl BotDb {
    /// Create a new room collection. Returns the generated id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn create_collection(&self, name: &str, description: Option<&str>) -> Result<i64> {
        self.engine
            .execute(
                "INSERT INTO room_collections (name, description, created_at) VALUES (?, ?, ?)",
                vec![
                    Value::String(name.to_string()),
                    description.map_or(Value::Null, |v| Value::String(v.to_string())),
                    Value::String(Utc::now().to_rfc3339()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("create_collection failed: {e}"))?;
        let id_rows = self
            .engine
            .execute("SELECT last_insert_rowid() AS id", vec![])
            .await
            .map_err(|e| anyhow::anyhow!("last_insert_rowid failed: {e}"))?;
        id_rows
            .rows
            .first()
            .and_then(|r| r.values.first())
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("create_collection: no rowid returned"))
    }

    /// Delete a room collection by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        self.engine
            .execute(
                "DELETE FROM room_collections WHERE id = ?",
                vec![Value::Number(id.into())],
            )
            .await
            .map_err(|e| anyhow::anyhow!("delete_collection failed: {e}"))?;
        Ok(())
    }

    /// List all room collections, ordered by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_collections(&self) -> Result<Vec<room_collection::Model>> {
        let rows = self
            .engine
            .execute("SELECT * FROM room_collections ORDER BY name ASC", vec![])
            .await
            .map_err(|e| anyhow::anyhow!("list_collections query failed: {e}"))?;
        rows.rows
            .iter()
            .map(room_collection::Model::from_row)
            .collect()
    }

    /// Add `platform`/`room_id` to a collection (idempotent).
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn add_to_collection(
        &self,
        collection_id: i64,
        platform: &str,
        room_id: &str,
    ) -> Result<()> {
        self.engine
            .execute(
                "INSERT OR IGNORE INTO room_collection_members \
                 (collection_id, platform, room_id) VALUES (?, ?, ?)",
                vec![
                    Value::Number(collection_id.into()),
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("add_to_collection failed: {e}"))?;
        Ok(())
    }

    /// Remove `platform`/`room_id` from a collection.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn remove_from_collection(
        &self,
        collection_id: i64,
        platform: &str,
        room_id: &str,
    ) -> Result<()> {
        self.engine
            .execute(
                "DELETE FROM room_collection_members \
                 WHERE collection_id = ? AND platform = ? AND room_id = ?",
                vec![
                    Value::Number(collection_id.into()),
                    Value::String(platform.to_string()),
                    Value::String(room_id.to_string()),
                ],
            )
            .await
            .map_err(|e| anyhow::anyhow!("remove_from_collection failed: {e}"))?;
        Ok(())
    }

    /// Return all members of a collection.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn rooms_in_collection(
        &self,
        collection_id: i64,
    ) -> Result<Vec<room_collection_member::Model>> {
        let rows = self
            .engine
            .execute(
                "SELECT * FROM room_collection_members WHERE collection_id = ?",
                vec![Value::Number(collection_id.into())],
            )
            .await
            .map_err(|e| anyhow::anyhow!("rooms_in_collection query failed: {e}"))?;
        rows.rows
            .iter()
            .map(room_collection_member::Model::from_row)
            .collect()
    }
}
