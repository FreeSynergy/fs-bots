// fs-bot/src/command.rs — BotCommand trait.

use async_trait::async_trait;

use crate::context::CommandContext;
use crate::response::BotResponse;
use crate::rights::Right;

/// A single bot command.
///
/// # Implementing a command
///
/// ```rust,ignore
/// use fs_bot::{BotCommand, BotResponse, CommandContext, Right};
/// use async_trait::async_trait;
///
/// pub struct PingCommand;
///
/// #[async_trait]
/// impl BotCommand for PingCommand {
///     fn name(&self) -> &str { "ping" }
///     fn description(&self) -> &str { "Check if the bot is alive" }
///     async fn execute(&self, _ctx: CommandContext) -> BotResponse {
///         BotResponse::text("pong")
///     }
/// }
/// ```
#[async_trait]
pub trait BotCommand: Send + Sync {
    /// Command name without prefix, e.g. `"ping"`, `"subscribe"`.
    fn name(&self) -> &str;

    /// Short human-readable description shown in /help output.
    fn description(&self) -> &'static str {
        ""
    }

    /// Minimum access level required to run this command.
    fn required_right(&self) -> Right {
        Right::None
    }

    /// Usage hint, e.g. `Some("subscribe <topic>")`. `None` means no args.
    fn usage(&self) -> Option<&str> {
        None
    }

    /// Execute the command and return a response.
    async fn execute(&self, ctx: CommandContext) -> BotResponse;
}
