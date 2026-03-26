// fs-bot/src/registry.rs — CommandRegistry for looking up and dispatching BotCommands.

use std::collections::HashMap;
use std::sync::Arc;

use crate::command::BotCommand;
use crate::context::CommandContext;
use crate::response::BotResponse;

// ── CommandRegistry ───────────────────────────────────────────────────────────

/// Registry mapping command names to [`BotCommand`] implementations.
#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn BotCommand>>,
}

impl CommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command. Wraps it in `Arc` internally.
    ///
    /// If a command with the same name already exists, it is replaced.
    pub fn register(&mut self, command: impl BotCommand + 'static) {
        let cmd = Arc::new(command);
        self.commands.insert(cmd.name().to_string(), cmd);
    }

    /// Look up a command by name (without prefix).
    pub fn get(&self, name: &str) -> Option<Arc<dyn BotCommand>> {
        self.commands.get(name).cloned()
    }

    /// Return all registered command names, sorted alphabetically.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.commands.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Return all registered commands as `(name, command)` pairs, sorted by name.
    pub fn all(&self) -> Vec<(&str, Arc<dyn BotCommand>)> {
        let mut pairs: Vec<_> = self
            .commands
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        pairs.sort_by_key(|(k, _)| *k);
        pairs
    }

    /// Dispatch a command context to the matching handler.
    ///
    /// Checks access level and calls `execute`. Returns `None` if the command
    /// is not found.
    pub async fn dispatch(&self, ctx: CommandContext) -> Option<BotResponse> {
        let cmd = self.get(&ctx.command)?;
        if ctx.caller_right < cmd.required_right() {
            return Some(BotResponse::error("Permission denied."));
        }
        Some(cmd.execute(ctx).await)
    }
}
