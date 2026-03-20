// GatekeeperHandler — queues chat.join_request events and notifies admins.

use async_trait::async_trait;
use chrono::Utc;
use fsn_bot::trigger::{TriggerAction, TriggerEvent, TriggerHandler};
use sqlx::SqlitePool;
use tracing::warn;

/// Listens on `chat.join_request` events.
///
/// Expected payload (JSON):
/// ```json
/// { "platform": "telegram", "room_id": "...", "user_id": "...", "user_name": "..." }
/// ```
pub struct GatekeeperHandler {
    pool: SqlitePool,
}

impl GatekeeperHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TriggerHandler for GatekeeperHandler {
    fn topics(&self) -> &[&str] {
        &["chat.join_request"]
    }

    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction> {
        let payload = &event.payload;

        let platform  = payload["platform"].as_str().unwrap_or("").to_owned();
        let room_id   = payload["room_id"].as_str().unwrap_or("").to_owned();
        let user_id   = payload["user_id"].as_str().unwrap_or("").to_owned();
        let user_name = payload["user_name"].as_str().unwrap_or(&user_id).to_owned();

        if platform.is_empty() || room_id.is_empty() || user_id.is_empty() {
            warn!("GatekeeperHandler: malformed payload: {:?}", payload);
            return vec![];
        }

        // Store join request
        let res = sqlx::query(
            "INSERT INTO join_requests
             (platform, room_id, user_id, status, created_at)
             VALUES (?, ?, ?, 'pending', ?)",
        )
        .bind(&platform)
        .bind(&room_id)
        .bind(&user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await;

        let request_id = match res {
            Ok(r) => r.last_insert_rowid(),
            Err(e) => {
                warn!("GatekeeperHandler DB error: {e}");
                return vec![];
            }
        };

        // Notify admins in the room
        let text = format!(
            "Join request #{request_id} from `{user_name}` ({user_id}).\n\
             Use /approve {request_id} or /deny {request_id}."
        );

        vec![TriggerAction::SendToRoom { platform, room_id, text }]
    }
}
