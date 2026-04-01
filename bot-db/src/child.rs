// child.rs — Child bot management methods for BotDb.

use anyhow::Result;
use chrono::Utc;
use fs_db::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

use crate::{entities::child_bot, BotDb};

impl BotDb {
    /// Register a new child bot (idempotent by name).
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn add_child_bot(&self, name: &str, bot_type: &str, data_dir: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        child_bot::Entity::insert(child_bot::ActiveModel {
            name: Set(name.to_string()),
            bot_type: Set(bot_type.to_string()),
            data_dir: Set(data_dir.to_string()),
            created_at: Set(Utc::now().to_rfc3339()),
            ..Default::default()
        })
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec(&self.conn)
        .await?;
        Ok(())
    }

    /// List all registered child bots, ordered by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn list_child_bots(&self) -> Result<Vec<child_bot::Model>> {
        Ok(child_bot::Entity::find()
            .order_by_asc(child_bot::Column::Name)
            .all(&self.conn)
            .await?)
    }

    /// Update the runtime status and PID of a child bot.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot is not found or the database write fails.
    pub async fn set_child_bot_status(
        &self,
        name: &str,
        status: &str,
        pid: Option<i64>,
    ) -> Result<()> {
        let mut model: child_bot::ActiveModel = child_bot::Entity::find()
            .filter(child_bot::Column::Name.eq(name))
            .one(&self.conn)
            .await?
            .ok_or_else(|| anyhow::anyhow!("child bot '{name}' not found"))?
            .into();
        model.status = Set(status.to_string());
        model.pid = Set(pid);
        model.update(&self.conn).await?;
        Ok(())
    }
}
