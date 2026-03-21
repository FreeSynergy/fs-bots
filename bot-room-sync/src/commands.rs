// Room-sync commands: /sync-start, /sync-stop, /sync-status

use std::sync::Arc;
use async_trait::async_trait;
use bot_db::BotDb;
use fs_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};

pub fn register_all(registry: &mut CommandRegistry, db: Arc<BotDb>) {
    registry.register(SyncStartCommand  { db: db.clone() });
    registry.register(SyncStopCommand   { db: db.clone() });
    registry.register(SyncStatusCommand { db });
}

// ── /sync-start ───────────────────────────────────────────────────────────────

struct SyncStartCommand { db: Arc<BotDb> }

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

        let src_platform = ctx.platform.as_str();
        let src_room     = ctx.room_id.as_str();

        match self.db.create_rule(src_platform, src_room, tgt_platform, tgt_room, "both", false).await {
            Ok(id) => BotResponse::text(format!(
                "Sync rule #{id} created: {src_platform}/{src_room} ↔ {tgt_platform}/{tgt_room}",
            )),
            Err(e) => BotResponse::error(format!("Error creating sync rule: {e}")),
        }
    }
}

// ── /sync-stop ────────────────────────────────────────────────────────────────

struct SyncStopCommand { db: Arc<BotDb> }

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

        let src_platform = ctx.platform.as_str();
        let src_room     = ctx.room_id.as_str();

        match self.db.disable_rule(src_platform, src_room, tgt_platform, tgt_room).await {
            Ok(true)  => BotResponse::text(format!("Sync with {tgt_platform}/{tgt_room} stopped.")),
            Ok(false) => BotResponse::error("No active sync rule found for those rooms."),
            Err(e)    => BotResponse::error(format!("Error: {e}")),
        }
    }
}

// ── /sync-status ──────────────────────────────────────────────────────────────

struct SyncStatusCommand { db: Arc<BotDb> }

#[async_trait]
impl BotCommand for SyncStatusCommand {
    fn name(&self) -> &str { "sync-status" }
    fn description(&self) -> &str { "List active sync rules for this room" }
    fn required_right(&self) -> Right { Right::Member }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let platform = ctx.platform.as_str();
        let room     = ctx.room_id.as_str();

        match self.db.active_rules_for(platform, room).await {
            Err(e)                        => BotResponse::error(format!("DB error: {e}")),
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
