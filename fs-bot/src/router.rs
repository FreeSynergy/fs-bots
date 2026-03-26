// fs-bot/src/router.rs — BotRouter bridges fs-bus IncomingMessage events to commands.

use std::sync::Arc;

use async_trait::async_trait;
use fs_bus::{BusError, Event, TopicHandler};
use fs_channel::{Channel, IncomingMessage};
use tracing::{debug, warn};

use crate::context::CommandContext;
use crate::registry::CommandRegistry;
use crate::rights::Right;

// ── PermissionResolver ────────────────────────────────────────────────────────

/// Maps sender IDs to access levels.
pub trait PermissionResolver: Send + Sync {
    fn resolve(&self, sender_id: &str) -> Right;
}

pub struct AllowAllPermissions;
impl PermissionResolver for AllowAllPermissions {
    fn resolve(&self, _sender_id: &str) -> Right {
        Right::Admin
    }
}

pub struct DenyAllPermissions;
impl PermissionResolver for DenyAllPermissions {
    fn resolve(&self, _sender_id: &str) -> Right {
        Right::None
    }
}

// ── BotRouter ─────────────────────────────────────────────────────────────────

/// Bridges fs-bus events to the bot [`CommandRegistry`].
///
/// Listens on `"channel.message.incoming"`, parses the command prefix, checks
/// the caller's right level, dispatches to the appropriate command, and sends
/// the [`BotResponse`] back via the channel adapter.
pub struct BotRouter {
    prefix: String,
    registry: CommandRegistry,
    channel: Arc<dyn Channel>,
    permissions: Arc<dyn PermissionResolver>,
    platform: String,
}

impl BotRouter {
    pub fn new(
        prefix: impl Into<String>,
        registry: CommandRegistry,
        channel: Arc<dyn Channel>,
        permissions: Arc<dyn PermissionResolver>,
        platform: impl Into<String>,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            registry,
            channel,
            permissions,
            platform: platform.into(),
        }
    }

    async fn handle_message(&self, msg: IncomingMessage) {
        let body = msg.body.trim();

        if !body.starts_with(&self.prefix) {
            return;
        }

        let after_prefix = body[self.prefix.len()..].trim();
        let mut tokens = after_prefix.split_whitespace();
        let cmd_name = match tokens.next() {
            Some(n) => n,
            None => return,
        };
        let args: Vec<String> = tokens.map(String::from).collect();

        debug!(sender = %msg.sender, command = %cmd_name, "bot command received");

        let caller_right = self.permissions.resolve(&msg.sender);

        let ctx = CommandContext::new(
            cmd_name,
            args,
            &self.platform,
            &msg.room_id,
            &msg.sender,
            caller_right,
        );

        let response = match self.registry.dispatch(ctx).await {
            Some(r) => r,
            None => return,
        };

        if let Some(msg_out) = response.into_channel_message() {
            if let Err(e) = self.channel.send(&msg.room_id, msg_out).await {
                warn!("BotRouter: send failed: {e}");
            }
        }
    }
}

#[async_trait]
impl TopicHandler for BotRouter {
    fn topic_pattern(&self) -> &str {
        "channel.message.incoming"
    }

    async fn handle(&self, event: &Event) -> Result<(), BusError> {
        let msg: IncomingMessage = event
            .parse_payload()
            .map_err(|e| BusError::handler(event.topic(), e.to_string()))?;
        self.handle_message(msg).await;
        Ok(())
    }
}
