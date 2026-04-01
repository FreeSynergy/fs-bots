// collection.rs — Room collection methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

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
        let result = room_collection::ActiveModel {
            name: Set(name.to_string()),
            description: Set(description.map(str::to_string)),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(result.id)
    }

    /// Delete a room collection by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn delete_collection(&self, id: i64) -> Result<()> {
        room_collection::Entity::delete_by_id(id)
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    /// List all room collections, ordered by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_collections(&self) -> Result<Vec<room_collection::Model>> {
        Ok(room_collection::Entity::find()
            .order_by_asc(room_collection::Column::Name)
            .all(&self.conn)
            .await?)
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
        use fs_db::sea_orm::sea_query::OnConflict;
        room_collection_member::Entity::insert(room_collection_member::ActiveModel {
            collection_id: Set(collection_id),
            platform: Set(platform.to_string()),
            room_id: Set(room_id.to_string()),
        })
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(&self.conn)
        .await?;
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
        room_collection_member::Entity::delete_many()
            .filter(room_collection_member::Column::CollectionId.eq(collection_id))
            .filter(room_collection_member::Column::Platform.eq(platform))
            .filter(room_collection_member::Column::RoomId.eq(room_id))
            .exec(&self.conn)
            .await?;
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
        Ok(room_collection_member::Entity::find()
            .filter(room_collection_member::Column::CollectionId.eq(collection_id))
            .all(&self.conn)
            .await?)
    }
}
