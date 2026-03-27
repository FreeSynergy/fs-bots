// /status — shows the bot's connected messengers and their feature support.

use async_trait::async_trait;
use fs_bot::rights::Right;
use fs_bot::{BotCommand, BotResponse, CommandContext};

pub struct StatusCommand;

#[async_trait]
impl BotCommand for StatusCommand {
    fn name(&self) -> &'static str {
        "status"
    }
    fn description(&self) -> &'static str {
        "Show bot status and connected messengers."
    }
    fn required_right(&self) -> Right {
        Right::Member
    }
    fn usage(&self) -> Option<&str> {
        Some("/status")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let platform = &ctx.platform;
        BotResponse::text(format!(
            "FreeSynergy Bot — online\nPlatform: {platform}\nType /help for available commands.",
        ))
    }
}
