// fs-bot/src/trigger.rs — TriggerHandler trait and related types.
//
// Trigger handlers process Bus events and return actions for the runtime to
// execute (send messages, DMs, etc.). They are registered in the TriggerEngine
// inside the bots crate and can read from BotDb.

use async_trait::async_trait;
use serde_json::Value;

// ── TriggerEvent ──────────────────────────────────────────────────────────────

/// A Bus event delivered to a trigger handler.
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    /// Bus topic, e.g. `"chat.message"` or `"calendar.event.upcoming"`.
    pub topic: String,
    /// Parsed JSON payload.
    pub payload: Value,
}

// ── TriggerAction ─────────────────────────────────────────────────────────────

/// An action the runtime should perform after a handler processes an event.
#[derive(Debug, Clone)]
pub enum TriggerAction {
    /// Send a text message to a room.
    SendToRoom {
        platform: String,
        room_id: String,
        text: String,
    },
    /// Send a direct message to a user.
    SendDm {
        platform: String,
        user_id: String,
        text: String,
    },
}

// ── TriggerHandler ────────────────────────────────────────────────────────────

/// A handler that subscribes to Bus topic patterns and produces actions.
#[async_trait]
pub trait TriggerHandler: Send + Sync {
    /// Topic patterns this handler subscribes to.
    ///
    /// Supports glob segments: `*` matches one segment, `**` matches the rest.
    fn topics(&self) -> &[&str];

    /// Process an event and return zero or more actions.
    async fn on_event(&self, event: TriggerEvent) -> Vec<TriggerAction>;
}
