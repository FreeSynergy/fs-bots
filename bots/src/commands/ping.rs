// /ping — responds with "Pong!" to confirm the bot is alive.

use async_trait::async_trait;
use fs_bot::rights::Right;
use fs_bot::{BotCommand, BotResponse, CommandContext};

pub struct PingCommand;

#[async_trait]
impl BotCommand for PingCommand {
    fn name(&self) -> &'static str {
        "ping"
    }
    fn description(&self) -> &'static str {
        "Check if the bot is alive."
    }
    fn required_right(&self) -> Right {
        Right::None
    }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        BotResponse::text("Pong!")
    }
}
