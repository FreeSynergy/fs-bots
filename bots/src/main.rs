#![deny(clippy::all, clippy::pedantic, warnings)]

// fs-bot-runtime — FreeSynergy bot instance entry point.
//
// Usage: fs-bot-runtime --config <path/to/bot.toml>

use anyhow::{Context, Result};
use fs_bot::CommandRegistry;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use fs_bots::{
    audit::AuditLog, config::BotInstanceConfig, db::BotDb, dispatcher::CommandDispatcher,
    runtime::BotRuntime, trigger::TriggerEngine,
};

mod commands;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config_path = std::env::args()
        .skip_while(|a| a != "--config")
        .nth(1)
        .unwrap_or_else(|| "bot.toml".to_owned());

    let config_str = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Cannot read config file '{config_path}'"))?;
    let config: BotInstanceConfig = toml::from_str(&config_str).context("Invalid bot.toml")?;

    let db_path = format!("{}/fs-botmanager.db", config.data_dir);
    let db = BotDb::open(&db_path)
        .await
        .with_context(|| format!("Cannot open database '{db_path}'"))?;
    let db = Arc::new(db);

    let audit = AuditLog::new(Arc::clone(&db));

    // Build command registry
    let mut registry = CommandRegistry::new();
    commands::register_all(&mut registry);

    // Register module commands + collect trigger handlers
    let mut trigger_handlers: Vec<Box<dyn fs_bot::TriggerHandler>> = Vec::new();
    trigger_handlers.extend(bot_broadcast::register(&mut registry, Arc::clone(&db)));
    trigger_handlers.extend(bot_gatekeeper::register(&mut registry, Arc::clone(&db)));
    trigger_handlers.extend(bot_calendar::register(&mut registry));
    trigger_handlers.extend(bot_control::register(&mut registry, Arc::clone(&db)));
    trigger_handlers.extend(bot_room_sync::register(&mut registry, Arc::clone(&db)));

    // Build trigger engine (returns action receiver)
    let (mut trigger, action_rx) = TriggerEngine::new(audit.clone());
    for h in trigger_handlers {
        trigger.register_boxed(h);
    }

    let dispatcher = CommandDispatcher::new(Arc::new(registry), audit.clone());
    let runtime = BotRuntime::new(
        config,
        dispatcher,
        trigger,
        action_rx,
        Arc::clone(&db),
        audit,
    );
    runtime.run().await;

    Ok(())
}
