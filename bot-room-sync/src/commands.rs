// Room-sync commands: /sync-start, /sync-stop, /sync-status

use async_trait::async_trait;
use fsn_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use sqlx::SqlitePool;
use crate::SyncDb;

pub fn register_all(registry: &mut CommandRegistry, pool: SqlitePool) {
    let db = SyncDb::new(pool);
    registry.register(SyncStartCommand { db: db.clone() });
    registry.register(SyncStopCommand  { db: db.clone() });
    registry.register(SyncStatusCommand { db });
}

// ── /sync-start ───────────────────────────────────────────────────────────────

struct SyncStartCommand { db: SyncDb }

#[async_trait]
impl BotCommand for SyncStartCommand {
    fn name(&self) -> &str { "sync-start" }
    fn description(&self) -> &str { "Start syncing this room with another room (same or cross-platform)" }
    fn required_right(&self) -> Right { Right::Admin }
    fn usage(&self) -> Option<&str> { Some("sync-start <target_platform> <target_room>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(tgt_platform) = ctx.arg0() else {
            return BotResponse::error("Usage: /sync-start <target_platform> <target_room>");
        };
        let Some(tgt_room) = ctx.args.get(1).map(|s| s.as_str()) else {
            return BotResponse::error("Usage: /sync-start <target_platform> <target_room>");
        };

        // Migrate schema in case this is the first use
        if let Err(e) = self.db.migrate().await {
            return BotResponse::error(format!("DB migration error: {e}"));
        }

        let src_platform = ctx.platform.label();
        let src_room     = ctx.room();

        match self.db.create_rule(src_platform, src_room.as_str(), tgt_platform, tgt_room, "both", false).await {
            Ok(id) => BotResponse::text(format!(
                "Sync rule #{id} created: {src_platform}/{} ↔ {tgt_platform}/{tgt_room}",
                src_room.as_str()
            )),
            Err(e) => BotResponse::error(format!("Error creating sync rule: {e}")),
        }
    }
}

// ── /sync-stop ────────────────────────────────────────────────────────────────

struct SyncStopCommand { db: SyncDb }

#[async_trait]
impl BotCommand for SyncStopCommand {
    fn name(&self) -> &str { "sync-stop" }
    fn description(&self) -> &str { "Stop syncing this room with another room" }
    fn required_right(&self) -> Right { Right::Admin }
    fn usage(&self) -> Option<&str> { Some("sync-stop <target_platform> <target_room>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(tgt_platform) = ctx.arg0() else {
            return BotResponse::error("Usage: /sync-stop <target_platform> <target_room>");
        };
        let Some(tgt_room) = ctx.args.get(1).map(|s| s.as_str()) else {
            return BotResponse::error("Usage: /sync-stop <target_platform> <target_room>");
        };

        let src_platform = ctx.platform.label();
        let src_room     = ctx.room();

        match self.db.disable_rule(src_platform, src_room.as_str(), tgt_platform, tgt_room).await {
            Ok(true)  => BotResponse::text(format!("Sync with {tgt_platform}/{tgt_room} stopped.")),
            Ok(false) => BotResponse::error("No active sync rule found for those rooms."),
            Err(e)    => BotResponse::error(format!("Error: {e}")),
        }
    }
}

// ── /sync-status ──────────────────────────────────────────────────────────────

struct SyncStatusCommand { db: SyncDb }

#[async_trait]
impl BotCommand for SyncStatusCommand {
    fn name(&self) -> &str { "sync-status" }
    fn description(&self) -> &str { "List active sync rules for this room" }
    fn required_right(&self) -> Right { Right::Member }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let platform = ctx.platform.label();
        let room     = ctx.room();

        if let Err(e) = self.db.migrate().await {
            return BotResponse::error(format!("DB migration error: {e}"));
        }

        match self.db.active_rules_for(platform, room.as_str()).await {
            Err(e)               => BotResponse::error(format!("DB error: {e}")),
            Ok(rules) if rules.is_empty() => BotResponse::text("No active sync rules for this room."),
            Ok(rules) => {
                let lines: Vec<String> = rules.iter().map(|r| {
                    let arrow = match r.direction.as_str() {
                        "to_target" => "→",
                        "to_source" => "←",
                        _           => "↔",
                    };
                    format!(
                        "  #{} {}/{} {} {}/{} (members={})",
                        r.id, r.source_platform, r.source_room,
                        arrow,
                        r.target_platform, r.target_room,
                        if r.sync_members { "yes" } else { "no" }
                    )
                }).collect();
                BotResponse::text(format!("Active sync rules:\n{}", lines.join("\n")))
            }
        }
    }
}
