// join.rs — Join request methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

use crate::{entities::join_request, BotDb};

impl BotDb {
    /// Queue a new join request. Returns the generated request id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn add_join_request(
        &self,
        platform: &str,
        room_id: &str,
        user_id: &str,
    ) -> Result<i64> {
        let result = join_request::ActiveModel {
            platform: Set(platform.to_string()),
            room_id: Set(room_id.to_string()),
            user_id: Set(user_id.to_string()),
            status: Set("pending".to_string()),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        }
        .insert(&self.conn)
        .await?;
        Ok(result.id)
    }

    /// Fetch a single join request by id.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_join_request(&self, id: i64) -> Result<Option<join_request::Model>> {
        Ok(join_request::Entity::find_by_id(id).one(&self.conn).await?)
    }

    /// List all pending join requests for `platform`/`room_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_pending_join_requests(
        &self,
        platform: &str,
        room_id: &str,
    ) -> Result<Vec<join_request::Model>> {
        Ok(join_request::Entity::find()
            .filter(join_request::Column::Platform.eq(platform))
            .filter(join_request::Column::RoomId.eq(room_id))
            .filter(join_request::Column::Status.eq("pending"))
            .order_by_asc(join_request::Column::CreatedAt)
            .all(&self.conn)
            .await?)
    }

    /// Update the status of a join request.
    ///
    /// # Errors
    ///
    /// Returns an error if the request is not found or the database write fails.
    pub async fn resolve_join_request(
        &self,
        id: i64,
        status: &str,
        iam_result: Option<&str>,
    ) -> Result<()> {
        let mut model: join_request::ActiveModel = join_request::Entity::find_by_id(id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("join request {id} not found"))?
            .into();
        model.status = Set(status.to_string());
        model.iam_result = Set(iam_result.map(str::to_string));
        model.resolved_at = Set(Some(Utc::now().to_rfc3339()));
        model.update(&self.conn).await?;
        Ok(())
    }
}
