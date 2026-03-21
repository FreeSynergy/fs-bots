// Broadcast commands: /subscribe, /unsubscribe, /subscriptions

use async_trait::async_trait;
use bot_db::BotDb;
use fs_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use std::sync::Arc;

pub fn register_all(registry: &mut CommandRegistry, db: Arc<BotDb>) {
    registry.register(SubscribeCommand   { db: db.clone() });
    registry.register(UnsubscribeCommand { db: db.clone() });
    registry.register(SubscriptionsCommand { db });
}

pub struct SubscribeCommand { pub db: Arc<BotDb> }

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
        match self.db.subscribe(&ctx.platform, &ctx.room_id, topic).await {
            Ok(_)  => BotResponse::text(format!("Subscribed to `{topic}`.")),
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

pub struct UnsubscribeCommand { pub db: Arc<BotDb> }

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
        match self.db.unsubscribe(&ctx.platform, &ctx.room_id, topic).await {
            Ok(_)  => BotResponse::text(format!("Unsubscribed from `{topic}`.")),
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

pub struct SubscriptionsCommand { pub db: Arc<BotDb> }

#[async_trait]
impl BotCommand for SubscriptionsCommand {
    fn name(&self) -> &str { "subscriptions" }
    fn description(&self) -> &str { "List active Bus topic subscriptions for this room" }
    fn required_right(&self) -> Right { Right::Member }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        match self.db.subscriptions_for_room(&ctx.platform, &ctx.room_id).await {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(topics) if topics.is_empty() => BotResponse::text("No active subscriptions."),
            Ok(topics) => BotResponse::text(format!(
                "Active subscriptions:\n{}",
                topics.iter().map(|t| format!("  • {t}")).collect::<Vec<_>>().join("\n")
            )),
        }
    }
}
