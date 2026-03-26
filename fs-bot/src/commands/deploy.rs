use crate::{BotCommand, BotResponse, CommandContext, Right};
use async_trait::async_trait;
use fs_bus::{Event, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct DeployCommand {
    bus: Arc<Mutex<Router>>,
    source: String,
}

impl DeployCommand {
    pub fn new(bus: Arc<Mutex<Router>>, source: impl Into<String>) -> Self {
        Self {
            bus,
            source: source.into(),
        }
    }
}

#[async_trait]
impl BotCommand for DeployCommand {
    fn name(&self) -> &str {
        "deploy"
    }
    fn description(&self) -> &str {
        "Trigger a deployment"
    }
    fn usage(&self) -> Option<&str> {
        Some("deploy <service> [<host>]")
    }
    fn required_right(&self) -> Right {
        Right::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(service) = ctx.args.first().cloned() else {
            return BotResponse::error("Usage: /deploy <service> [<host>]");
        };
        let host = ctx.args.get(1).cloned();

        let payload = serde_json::json!({
            "service": service,
            "host": host,
            "requested_by": ctx.sender,
        });

        let event = match Event::new("deploy.requested", &self.source, payload) {
            Ok(e) => e,
            Err(e) => return BotResponse::error(format!("Event error: {e}")),
        };

        self.bus.lock().await.dispatch(&event).await;
        info!(service = %service, sender = %ctx.sender, "deploy requested via bot");

        BotResponse::text(match &host {
            Some(h) => format!("Deploy of **{service}** on **{h}** requested."),
            None => format!("Deploy of **{service}** requested."),
        })
    }
}
