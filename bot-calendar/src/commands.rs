// Calendar commands: /termine

use async_trait::async_trait;
use fs_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};

pub fn register_all(registry: &mut CommandRegistry) {
    registry.register(TermineCommand);
}

// ── /termine ──────────────────────────────────────────────────────────────────

/// Shows upcoming events from the `FreeSynergy` Desktop Calendar.
pub struct TermineCommand;

#[async_trait]
impl BotCommand for TermineCommand {
    fn name(&self) -> &'static str {
        "termine"
    }
    fn description(&self) -> &'static str {
        "Show upcoming calendar events"
    }
    fn required_right(&self) -> Right {
        Right::Member
    }

    async fn execute(&self, _ctx: CommandContext) -> BotResponse {
        // TODO Phase E: query calendar.event.list via Bus → Desktop-Calendar service
        BotResponse::text(
            "Calendar integration will be available once a Desktop Calendar service \
             is connected to the Bus (Phase E).",
        )
    }
}
