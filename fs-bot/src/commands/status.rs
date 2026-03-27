use crate::{BotCommand, BotResponse, CommandContext, Right};
use async_trait::async_trait;

pub struct StatusCommand {
    provider: Box<dyn StatusProvider>,
}

pub trait StatusProvider: Send + Sync {
    fn status(&self) -> String;
}

pub struct DefaultStatusProvider;

impl StatusProvider for DefaultStatusProvider {
    fn status(&self) -> String {
        "All systems operational.".to_string()
    }
}

impl StatusCommand {
    #[must_use]
    pub fn new(provider: Box<dyn StatusProvider>) -> Self {
        Self { provider }
    }
}

impl Default for StatusCommand {
    fn default() -> Self {
        Self::new(Box::new(DefaultStatusProvider))
    }
}

#[async_trait]
impl BotCommand for StatusCommand {
    fn name(&self) -> &'static str {
        "status"
    }
    fn description(&self) -> &'static str {
        "Show system status"
    }
    fn usage(&self) -> Option<&str> {
        Some("status")
    }
    fn required_right(&self) -> Right {
        Right::Member
    }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        BotResponse::text(self.provider.status())
    }
}
