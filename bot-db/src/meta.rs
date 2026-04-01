// meta.rs — Bot metadata key/value store methods for BotDb.

use anyhow::Result;
use fs_db::sea_orm::{ActiveValue::Set, EntityTrait};

use crate::{entities::bot_meta, BotDb};

impl BotDb {
    /// Read a metadata key, returning `None` if not set.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_meta(&self, key: &str) -> Result<Option<String>> {
        Ok(bot_meta::Entity::find_by_id(key.to_string())
            .one(&self.conn)
            .await?
            .map(|m| m.value))
    }

    /// Write (upsert) a metadata key/value pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        bot_meta::Entity::insert(bot_meta::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
        })
        .on_conflict(
            OnConflict::column(bot_meta::Column::Key)
                .update_column(bot_meta::Column::Value)
                .to_owned(),
        )
        .exec(&self.conn)
        .await?;
        Ok(())
    }
}
