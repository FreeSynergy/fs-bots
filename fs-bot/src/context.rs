// fs-bot/src/context.rs — CommandContext passed to every BotCommand.

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
}

impl CommandContext {
    /// Create a new context.
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
        }
    }

    /// Room ID as an owned `String` (useful when `.as_str()` is needed downstream).
    pub fn room(&self) -> String {
        self.room_id.clone()
    }

    /// First argument, if any.
    pub fn arg0(&self) -> Option<&str> {
        self.args.first().map(|s| s.as_str())
    }
}
