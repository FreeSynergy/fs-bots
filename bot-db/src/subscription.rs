// subscription.rs — Room subscription methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

use crate::{entities::subscription, BotDb};

impl BotDb {
    /// Subscribe `platform`/`room_id` to `topic` (idempotent).
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn subscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        let model = subscription::ActiveModel {
            platform: Set(platform.to_string()),
            room_id: Set(room_id.to_string()),
            topic: Set(topic.to_string()),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        };
        subscription::Entity::insert(model)
            .on_conflict(OnConflict::new().do_nothing().to_owned())
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    /// Remove the subscription of `platform`/`room_id` from `topic`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn unsubscribe(&self, platform: &str, room_id: &str, topic: &str) -> Result<()> {
        subscription::Entity::delete_many()
            .filter(subscription::Column::Platform.eq(platform))
            .filter(subscription::Column::RoomId.eq(room_id))
            .filter(subscription::Column::Topic.eq(topic))
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    /// Return all topic names subscribed to by `platform`/`room_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn subscriptions_for_room(
        &self,
        platform: &str,
        room_id: &str,
    ) -> Result<Vec<String>> {
        Ok(subscription::Entity::find()
            .filter(subscription::Column::Platform.eq(platform))
            .filter(subscription::Column::RoomId.eq(room_id))
            .all(&self.conn)
            .await?
            .into_iter()
            .map(|r| r.topic)
            .collect())
    }

    /// All (platform, `room_id`) pairs subscribed to the given topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn subscriptions_for_room_by_topic(
        &self,
        topic: &str,
    ) -> Result<Vec<(String, String)>> {
        Ok(subscription::Entity::find()
            .filter(subscription::Column::Topic.eq(topic))
            .all(&self.conn)
            .await?
            .into_iter()
            .map(|r| (r.platform, r.room_id))
            .collect())
    }
}
