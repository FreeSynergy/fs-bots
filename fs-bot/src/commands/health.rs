use crate::{BotCommand, BotResponse, CommandContext, Right};
use async_trait::async_trait;

pub struct HealthQueryCommand {
    provider: Box<dyn HealthQueryProvider>,
}

pub trait HealthQueryProvider: Send + Sync {
    fn query(&self, target: Option<&str>) -> String;
}

pub struct StubHealthProvider;

impl HealthQueryProvider for StubHealthProvider {
    fn query(&self, target: Option<&str>) -> String {
        match target {
            Some(t) => format!("✓ {t}: healthy"),
            None => "✓ All services healthy.".to_string(),
        }
    }
}

impl HealthQueryCommand {
    pub fn new(provider: Box<dyn HealthQueryProvider>) -> Self {
        Self { provider }
    }

    pub fn stub() -> Self {
        Self::new(Box::new(StubHealthProvider))
    }
}

#[async_trait]
impl BotCommand for HealthQueryCommand {
    fn name(&self) -> &str {
        "health"
    }
    fn description(&self) -> &str {
        "Show service health"
    }
    fn usage(&self) -> Option<&str> {
        Some("health [<service>]")
    }
    fn required_right(&self) -> Right {
        Right::Member
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let target = ctx.args.first().map(String::as_str);
        BotResponse::text(self.provider.query(target))
    }
}
