// CalendarHandler — forwards calendar.event.* Bus events to rooms as reminders.

use async_trait::async_trait;
use fs_bot::trigger::{TriggerAction, TriggerEvent, TriggerHandler};
use tracing::warn;

/// Listens on `calendar.event.*` and sends event reminders.
///
/// Expected payload for `calendar.event.upcoming` (JSON):
/// ```json
/// {
///   "title": "Team Meeting",
///   "time": "2026-03-20T14:00:00Z",
///   "location": "Room 3",
///   "rooms": [
///     { "platform": "telegram", "room_id": "..." }
///   ],
///   "participants": [
///     { "platform": "telegram", "user_id": "..." }
///   ]
/// }
/// ```
pub struct CalendarHandler;

#[async_trait]
impl TriggerHandler for CalendarHandler {
    fn topics(&self) -> &[&str] {
        &["calendar.event.*"]
    }

    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction> {
        let payload = &event.payload;

        let title    = payload["title"].as_str().unwrap_or("Unnamed event");
        let time     = payload["time"].as_str().unwrap_or("(time unknown)");
        let location = payload["location"].as_str();

        let text = if let Some(loc) = location {
            format!("📅 Upcoming: **{title}**\n🕐 {time}\n📍 {loc}")
        } else {
            format!("📅 Upcoming: **{title}**\n🕐 {time}")
        };

        let mut actions: Vec<TriggerAction> = Vec::new();

        // Notify rooms
        if let Some(rooms) = payload["rooms"].as_array() {
            for room in rooms {
                let platform = room["platform"].as_str().unwrap_or("").to_owned();
                let room_id  = room["room_id"].as_str().unwrap_or("").to_owned();
                if platform.is_empty() || room_id.is_empty() {
                    warn!("CalendarHandler: malformed room entry: {:?}", room);
                    continue;
                }
                actions.push(TriggerAction::SendToRoom {
                    platform,
                    room_id,
                    text: text.clone(),
                });
            }
        }

        // DM participants
        if let Some(participants) = payload["participants"].as_array() {
            let dm_text = format!("{text}\nThis event includes you.");
            for p in participants {
                let platform = p["platform"].as_str().unwrap_or("").to_owned();
                let user_id  = p["user_id"].as_str().unwrap_or("").to_owned();
                if platform.is_empty() || user_id.is_empty() {
                    continue;
                }
                actions.push(TriggerAction::SendDm {
                    platform,
                    user_id,
                    text: dm_text.clone(),
                });
            }
        }

        actions
    }
}
