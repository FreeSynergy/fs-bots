// poll.rs — Poll state (message offset tracking) methods for BotDb.

use anyhow::Result;
use fs_db::sea_orm::{ActiveValue::Set, EntityTrait};

use crate::{entities::poll_state, BotDb};

impl BotDb {
    /// Return the last poll offset for `platform`/`room_id`, or 0 if not set.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn get_offset(&self, platform: &str, room_id: &str) -> Result<u64> {
        let row = poll_state::Entity::find_by_id((platform.to_string(), room_id.to_string()))
            .one(&self.conn)
            .await?;
        Ok(row.map_or(0, |r| r.last_offset.cast_unsigned()))
    }

    /// Persist the poll offset for `platform`/`room_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database write fails.
    pub async fn set_offset(&self, platform: &str, room_id: &str, offset: u64) -> Result<()> {
        use fs_db::sea_orm::sea_query::OnConflict;
        let model = poll_state::ActiveModel {
            platform: Set(platform.to_string()),
            room_id: Set(room_id.to_string()),
            last_offset: Set(offset.cast_signed()),
        };
        poll_state::Entity::insert(model)
            .on_conflict(
                OnConflict::columns([poll_state::Column::Platform, poll_state::Column::RoomId])
                    .update_column(poll_state::Column::LastOffset)
                    .to_owned(),
            )
            .exec(&self.conn)
            .await?;
        Ok(())
    }
}
