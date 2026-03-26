// fs-bot/src/error.rs — Error types for the bot framework.

use thiserror::Error;

/// Errors produced by the bot command system.
#[derive(Error, Debug)]
pub enum BotError {
    /// Command was not found in the registry.
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    /// The sender lacks the required permission to execute the command.
    #[error("Permission denied for command '{command}' (requires '{required}')")]
    PermissionDenied { command: String, required: String },

    /// The command received invalid or missing arguments.
    #[error("Invalid arguments for '{command}': {message}")]
    InvalidArgs { command: String, message: String },

    /// A channel send/receive operation failed.
    #[error("Channel error: {0}")]
    Channel(String),

    /// Catch-all for unexpected failures.
    #[error("Internal bot error: {0}")]
    Internal(String),
}

impl BotError {
    /// Convenience constructor for unknown command errors.
    pub fn unknown(command: impl Into<String>) -> Self {
        Self::UnknownCommand(command.into())
    }

    /// Convenience constructor for permission denied errors.
    pub fn permission_denied(command: impl Into<String>, required: impl Into<String>) -> Self {
        Self::PermissionDenied {
            command: command.into(),
            required: required.into(),
        }
    }

    /// Convenience constructor for invalid argument errors.
    pub fn invalid_args(command: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidArgs {
            command: command.into(),
            message: message.into(),
        }
    }

    /// Convenience constructor for internal errors.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
