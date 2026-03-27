// Control commands: /bots, /bot-create, /bot-status, /bot-logs

use async_trait::async_trait;
use bot_db::BotDb;
use fs_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use std::sync::Arc;

pub fn register_all(registry: &mut CommandRegistry, db: Arc<BotDb>) {
    registry.register(BotsCommand { db: db.clone() });
    registry.register(BotCreateCommand { db: db.clone() });
    registry.register(BotStatusCommand { db: db.clone() });
    registry.register(BotLogsCommand { db });
}

pub struct BotsCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for BotsCommand {
    fn name(&self) -> &'static str {
        "bots"
    }
    fn description(&self) -> &'static str {
        "List all managed child bot instances"
    }
    fn required_right(&self) -> Right {
        Right::Admin
    }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        match self.db.list_child_bots().await {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(bots) if bots.is_empty() => {
                BotResponse::text("No child bots registered. Use /bot-create <name> <type>.")
            }
            Ok(bots) => {
                let lines: Vec<String> = bots
                    .iter()
                    .map(|b| {
                        let pid_str = b.pid.map(|p| format!(" (pid {p})")).unwrap_or_default();
                        format!("  • {} [{}] — {}{}", b.name, b.bot_type, b.status, pid_str)
                    })
                    .collect();
                BotResponse::text(format!("Child bots:\n{}", lines.join("\n")))
            }
        }
    }
}

pub struct BotCreateCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for BotCreateCommand {
    fn name(&self) -> &'static str {
        "bot-create"
    }
    fn description(&self) -> &'static str {
        "Register a new child bot instance"
    }
    fn required_right(&self) -> Right {
        Right::Admin
    }
    fn usage(&self) -> Option<&str> {
        Some("bot-create <name> <type>")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        if ctx.args.len() < 2 {
            return BotResponse::error("Usage: /bot-create <name> <type>");
        }
        let name = &ctx.args[0];
        let bot_type = &ctx.args[1];
        let data_dir = format!("/var/lib/fs-bots/{name}");
        match self.db.add_child_bot(name, bot_type, &data_dir).await {
            Ok(()) => BotResponse::text(format!(
                "Child bot `{name}` ({bot_type}) registered.\nData dir: {data_dir}"
            )),
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

pub struct BotStatusCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for BotStatusCommand {
    fn name(&self) -> &'static str {
        "bot-status"
    }
    fn description(&self) -> &'static str {
        "Show status of a child bot"
    }
    fn required_right(&self) -> Right {
        Right::Admin
    }
    fn usage(&self) -> Option<&str> {
        Some("bot-status <name>")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(name) = ctx.arg0() else {
            return BotResponse::error("Usage: /bot-status <name>");
        };
        match self.db.list_child_bots().await {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(bots) => match bots.into_iter().find(|b| b.name == name) {
                None => BotResponse::error(format!("Bot `{name}` not found.")),
                Some(b) => {
                    let pid_line = b
                        .pid
                        .map(|p| format!("\nPID:      {p}"))
                        .unwrap_or_default();
                    BotResponse::text(format!(
                        "Bot:      {}\nType:     {}\nStatus:   {}{}\nData:     {}\nCreated:  {}",
                        b.name, b.bot_type, b.status, pid_line, b.data_dir, b.created_at
                    ))
                }
            },
        }
    }
}

pub struct BotLogsCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for BotLogsCommand {
    fn name(&self) -> &'static str {
        "bot-logs"
    }
    fn description(&self) -> &'static str {
        "Show recent audit log entries"
    }
    fn required_right(&self) -> Right {
        Right::Admin
    }
    fn usage(&self) -> Option<&str> {
        Some("bot-logs [limit]")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let limit: u64 = ctx
            .arg0()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_u64)
            .min(50);
        match self.db.recent_audit(limit).await {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(entries) if entries.is_empty() => BotResponse::text("Audit log is empty."),
            Ok(entries) => {
                let lines: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        format!(
                            "[{}] {}/{} — {} → {}",
                            e.created_at, e.actor_type, e.actor_id, e.action, e.result
                        )
                    })
                    .collect();
                BotResponse::text(format!("Last {limit} audit entries:\n{}", lines.join("\n")))
            }
        }
    }
}
