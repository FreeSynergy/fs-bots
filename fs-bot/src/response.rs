// fs-bot/src/response.rs — BotResponse type returned by BotCommand::execute.

/// The response a bot command produces.
///
/// The runtime sends the appropriate message(s) back through the channel adapter.
#[derive(Debug, Clone)]
pub enum BotResponse {
    /// A plain text reply to the originating room.
    Text(String),
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

    /// Convert into a `ChannelMessage` to send, or `None` for `Silent`.
    pub fn into_channel_message(self) -> Option<fs_channel::ChannelMessage> {
        match self {
            Self::Text(text) => Some(fs_channel::ChannelMessage::text(text)),
            Self::Error(text) => Some(fs_channel::ChannelMessage::text(format!("Error: {text}"))),
            Self::Silent => None,
        }
    }
}
