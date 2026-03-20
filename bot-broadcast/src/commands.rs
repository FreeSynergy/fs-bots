// Broadcast commands: /subscribe, /unsubscribe, /subscriptions

use async_trait::async_trait;
use chrono::Utc;
use fsn_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use sqlx::SqlitePool;

pub fn register_all(registry: &mut CommandRegistry, pool: SqlitePool) {
    registry.register(SubscribeCommand   { pool: pool.clone() });
    registry.register(UnsubscribeCommand { pool: pool.clone() });
    registry.register(SubscriptionsCommand { pool });
}

// ── /subscribe ────────────────────────────────────────────────────────────────

pub struct SubscribeCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for SubscribeCommand {
    fn name(&self) -> &str { "subscribe" }
    fn description(&self) -> &str { "Subscribe this room to a Bus topic" }
    fn required_right(&self) -> Right { Right::Operator }
    fn usage(&self) -> Option<&str> { Some("subscribe <topic>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(topic) = ctx.arg0() else {
            return BotResponse::error("Usage: /subscribe <topic>");
        };
        let platform = ctx.platform.label();
        let room_id  = ctx.room().as_str();

        match sqlx::query(
            "INSERT OR IGNORE INTO subscriptions (platform, room_id, topic, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(platform)
        .bind(room_id)
        .bind(topic)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        {
            Ok(_)  => BotResponse::text(format!("Subscribed to `{topic}`.")),
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

// ── /unsubscribe ──────────────────────────────────────────────────────────────

pub struct UnsubscribeCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for UnsubscribeCommand {
    fn name(&self) -> &str { "unsubscribe" }
    fn description(&self) -> &str { "Unsubscribe this room from a Bus topic" }
    fn required_right(&self) -> Right { Right::Operator }
    fn usage(&self) -> Option<&str> { Some("unsubscribe <topic>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(topic) = ctx.arg0() else {
            return BotResponse::error("Usage: /unsubscribe <topic>");
        };
        let platform = ctx.platform.label();
        let room_id  = ctx.room().as_str();

        match sqlx::query(
            "DELETE FROM subscriptions WHERE platform = ? AND room_id = ? AND topic = ?",
        )
        .bind(platform)
        .bind(room_id)
        .bind(topic)
        .execute(&self.pool)
        .await
        {
            Ok(_)  => BotResponse::text(format!("Unsubscribed from `{topic}`.")),
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

// ── /subscriptions ────────────────────────────────────────────────────────────

pub struct SubscriptionsCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for SubscriptionsCommand {
    fn name(&self) -> &str { "subscriptions" }
    fn description(&self) -> &str { "List active Bus topic subscriptions for this room" }
    fn required_right(&self) -> Right { Right::Member }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let platform = ctx.platform.label();
        let room_id  = ctx.room().as_str();

        let rows = sqlx::query(
            "SELECT topic FROM subscriptions WHERE platform = ? AND room_id = ? ORDER BY topic",
        )
        .bind(platform)
        .bind(room_id)
        .fetch_all(&self.pool)
        .await;

        match rows {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(rows) if rows.is_empty() => {
                BotResponse::text("No active subscriptions for this room.")
            }
            Ok(rows) => {
                use sqlx::Row;
                let topics: Vec<String> = rows
                    .iter()
                    .map(|r| r.get::<String, _>(0))
                    .collect();
                BotResponse::text(format!(
                    "Active subscriptions:\n{}",
                    topics.iter().map(|t| format!("  • {t}")).collect::<Vec<_>>().join("\n")
                ))
            }
        }
    }
}
