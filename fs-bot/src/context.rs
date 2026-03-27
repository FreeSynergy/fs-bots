// fs-bot/src/context.rs — CommandContext passed to every BotCommand.

use fs_channel::IncomingMessage;

use crate::rights::Right;

/// Runtime context available to every [`BotCommand`](crate::command::BotCommand).
///
/// Contains the caller's identity, platform, room, and parsed arguments.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Parsed command name (without prefix).
    pub command: String,
    /// Arguments after the command name.
    pub args: Vec<String>,
    /// Platform label, e.g. `"matrix"` or `"telegram"`.
    pub platform: String,
    /// Room or chat ID the message came from.
    pub room_id: String,
    /// Sender identifier (user ID, username, etc.).
    pub sender: String,
    /// Resolved access level of the caller.
    pub caller_right: Right,
    /// Resolved `FreeSynergy` user ID (set by IAM bridge in Phase P).
    pub fs_user_id: Option<String>,
    /// Arbitrary extra data for command-specific metadata.
    pub extra: serde_json::Value,
    /// Original incoming message, if available.
    pub message: Option<IncomingMessage>,
}

impl CommandContext {
    /// Create a new context (`fs_user_id/extra/message` default to None/Null/None).
    pub fn new(
        command: impl Into<String>,
        args: Vec<String>,
        platform: impl Into<String>,
        room_id: impl Into<String>,
        sender: impl Into<String>,
        caller_right: Right,
    ) -> Self {
        Self {
            command: command.into(),
            args,
            platform: platform.into(),
            room_id: room_id.into(),
            sender: sender.into(),
            caller_right,
            fs_user_id: None,
            extra: serde_json::Value::Null,
            message: None,
        }
    }

    /// Room ID as an owned `String` (useful when `.as_str()` is needed downstream).
    #[must_use]
    pub fn room(&self) -> String {
        self.room_id.clone()
    }

    /// First argument, if any.
    #[must_use]
    pub fn arg0(&self) -> Option<&str> {
        self.args.first().map(String::as_str)
    }
}
