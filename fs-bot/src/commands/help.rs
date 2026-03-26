use crate::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use async_trait::async_trait;
use std::sync::Arc;

pub struct HelpCommand {
    registry: Arc<CommandRegistry>,
    prefix: String,
}

impl HelpCommand {
    pub fn new(registry: Arc<CommandRegistry>, prefix: impl Into<String>) -> Self {
        Self {
            registry,
            prefix: prefix.into(),
        }
    }
}

#[async_trait]
impl BotCommand for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    fn description(&self) -> &str {
        "List all available commands"
    }
    fn required_right(&self) -> Right {
        Right::None
    }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        let mut lines = vec!["Available commands:".to_string()];
        for (_, cmd) in self.registry.all() {
            let usage = cmd.usage().unwrap_or(cmd.name());
            lines.push(format!(
                "  {}{} — {}",
                self.prefix,
                usage,
                cmd.description()
            ));
        }
        BotResponse::text(lines.join("\n"))
    }
}
