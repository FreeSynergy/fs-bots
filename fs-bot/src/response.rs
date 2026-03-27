// fs-bot/src/response.rs — BotResponse type returned by BotCommand::execute.

use fs_channel::{ChannelMessage, MessageFormat, RoomId, UserId};

/// The response a bot command produces.
///
/// The runtime sends the appropriate message(s) back through the channel adapter.
#[derive(Debug, Clone)]
pub enum BotResponse {
    /// A plain text reply to the originating room.
    Text(String),
    /// A formatted message to a specific room (or the originating room if `None`).
    Message {
        /// Target room; `None` means reply to the originating room.
        room: Option<RoomId>,
        /// Message text.
        text: String,
        /// Rendering format.
        format: MessageFormat,
    },
    /// An interactive menu sent to a specific room.
    Menu {
        /// Target room; `None` means the originating room.
        room: Option<RoomId>,
        /// Prompt text.
        text: String,
        /// Button labels.
        buttons: Vec<String>,
    },
    /// A direct message to a specific user.
    Dm {
        /// Target user.
        user: UserId,
        /// Message text.
        text: String,
    },
    /// Multiple responses sent in sequence.
    Many(Vec<BotResponse>),
    /// An error reply to the originating room (shown prefixed with "Error:").
    Error(String),
    /// No reply (command handled silently).
    Silent,
}

impl BotResponse {
    /// Create a plain-text response.
    pub fn text(msg: impl Into<String>) -> Self {
        Self::Text(msg.into())
    }

    /// Create an error response.
    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }

    /// Convert into a `ChannelMessage` to send via the legacy `Channel` trait, or `None` for
    /// variants that require `BotChannel` (menus, DMs) or are silent.
    #[must_use]
    pub fn into_channel_message(self) -> Option<ChannelMessage> {
        match self {
            Self::Message { text, format, .. } => Some(match format {
                MessageFormat::Markdown => ChannelMessage::markdown(text),
                _ => ChannelMessage::text(text),
            }),
            Self::Error(text) => Some(ChannelMessage::text(format!("Error: {text}"))),
            Self::Text(text) | Self::Menu { text, .. } | Self::Dm { text, .. } => {
                Some(ChannelMessage::text(text))
            }
            Self::Many(_) | Self::Silent => None,
        }
    }
}
