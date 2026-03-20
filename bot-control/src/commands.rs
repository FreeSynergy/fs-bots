// Control commands: /bots, /bot-create, /bot-status, /bot-logs

use async_trait::async_trait;
use chrono::Utc;
use fsn_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use sqlx::{Row, SqlitePool};

pub fn register_all(registry: &mut CommandRegistry, pool: SqlitePool) {
    registry.register(BotsCommand      { pool: pool.clone() });
    registry.register(BotCreateCommand { pool: pool.clone() });
    registry.register(BotStatusCommand { pool: pool.clone() });
    registry.register(BotLogsCommand   { pool });
}

// ── /bots ─────────────────────────────────────────────────────────────────────

pub struct BotsCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for BotsCommand {
    fn name(&self) -> &str { "bots" }
    fn description(&self) -> &str { "List all managed child bot instances" }
    fn required_right(&self) -> Right { Right::Admin }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        let rows = sqlx::query(
            "SELECT name, bot_type, status, pid FROM child_bots ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await;

        match rows {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(rows) if rows.is_empty() => {
                BotResponse::text("No child bots registered. Use /bot-create <name> <type>.")
            }
            Ok(rows) => {
                let lines: Vec<String> = rows
                    .iter()
                    .map(|r| {
                        let name:   String       = r.get(0);
                        let kind:   String       = r.get(1);
                        let status: String       = r.get(2);
                        let pid:    Option<i64>  = r.get(3);
                        let pid_str = pid.map(|p| format!(" (pid {p})")).unwrap_or_default();
                        format!("  • {name} [{kind}] — {status}{pid_str}")
                    })
                    .collect();
                BotResponse::text(format!("Child bots:\n{}", lines.join("\n")))
            }
        }
    }
}

// ── /bot-create ───────────────────────────────────────────────────────────────

pub struct BotCreateCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for BotCreateCommand {
    fn name(&self) -> &str { "bot-create" }
    fn description(&self) -> &str { "Register a new child bot instance" }
    fn required_right(&self) -> Right { Right::Admin }
    fn usage(&self) -> Option<&str> { Some("bot-create <name> <type>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        if ctx.args.len() < 2 {
            return BotResponse::error("Usage: /bot-create <name> <type>\nExample: /bot-create support broadcast");
        }
        let name     = &ctx.args[0];
        let bot_type = &ctx.args[1];
        let data_dir = format!("/var/lib/fsn-bots/{name}");

        let res = sqlx::query(
            "INSERT OR IGNORE INTO child_bots (name, bot_type, data_dir, status, created_at)
             VALUES (?, ?, ?, 'stopped', ?)",
        )
        .bind(name)
        .bind(bot_type)
        .bind(&data_dir)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await;

        match res {
            Ok(r) if r.rows_affected() == 0 => {
                BotResponse::error(format!("Bot `{name}` already exists."))
            }
            Ok(_) => BotResponse::text(format!(
                "Child bot `{name}` ({bot_type}) registered.\n\
                 Data dir: {data_dir}\n\
                 Process management will be added in Phase N11."
            )),
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

// ── /bot-status ───────────────────────────────────────────────────────────────

pub struct BotStatusCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for BotStatusCommand {
    fn name(&self) -> &str { "bot-status" }
    fn description(&self) -> &str { "Show status of a child bot" }
    fn required_right(&self) -> Right { Right::Admin }
    fn usage(&self) -> Option<&str> { Some("bot-status <name>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(name) = ctx.arg0() else {
            return BotResponse::error("Usage: /bot-status <name>");
        };

        let row = sqlx::query(
            "SELECT name, bot_type, status, pid, data_dir, created_at
             FROM child_bots WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await;

        match row {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(None) => BotResponse::error(format!("Bot `{name}` not found.")),
            Ok(Some(r)) => {
                let bot_name: String       = r.get(0);
                let kind:     String       = r.get(1);
                let status:   String       = r.get(2);
                let pid:      Option<i64>  = r.get(3);
                let data_dir: String       = r.get(4);
                let created:  String       = r.get(5);
                let pid_line = pid
                    .map(|p| format!("\nPID:      {p}"))
                    .unwrap_or_default();
                BotResponse::text(format!(
                    "Bot:      {bot_name}\nType:     {kind}\nStatus:   {status}{pid_line}\nData:     {data_dir}\nCreated:  {created}"
                ))
            }
        }
    }
}

// ── /bot-logs ─────────────────────────────────────────────────────────────────

pub struct BotLogsCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for BotLogsCommand {
    fn name(&self) -> &str { "bot-logs" }
    fn description(&self) -> &str { "Show recent audit log entries" }
    fn required_right(&self) -> Right { Right::Admin }
    fn usage(&self) -> Option<&str> { Some("bot-logs [limit]") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let limit: i64 = ctx.arg0()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10)
            .min(50);

        let rows = sqlx::query(
            "SELECT actor_type, actor_id, action, result, created_at
             FROM audit_log ORDER BY id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await;

        match rows {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(rows) if rows.is_empty() => BotResponse::text("Audit log is empty."),
            Ok(rows) => {
                let lines: Vec<String> = rows
                    .iter()
                    .map(|r| {
                        let actor_type: String = r.get(0);
                        let actor_id:   String = r.get(1);
                        let action:     String = r.get(2);
                        let result:     String = r.get(3);
                        let ts:         String = r.get(4);
                        format!("[{ts}] {actor_type}/{actor_id} — {action} → {result}")
                    })
                    .collect();
                BotResponse::text(format!(
                    "Last {limit} audit entries:\n{}",
                    lines.join("\n")
                ))
            }
        }
    }
}
