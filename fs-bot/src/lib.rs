#![deny(clippy::all, clippy::pedantic, warnings)]

//! Bot command framework for `FreeSynergy`.
//!
//! Provides:
//! - [`BotCommand`] trait and [`CommandRegistry`] for dispatching commands
//! - [`BotResponse`] as the command return type
//! - [`Right`] for access-level checking
//! - [`trigger`] module with [`TriggerHandler`], [`TriggerEvent`], [`TriggerAction`]
//! - Standard built-in commands

pub mod command;
pub mod commands;
pub mod context;
pub mod error;
pub mod registry;
pub mod response;
pub mod rights;
pub mod router;
pub mod trigger;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use command::BotCommand;
pub use context::CommandContext;
pub use error::BotError;
pub use registry::CommandRegistry;
pub use response::BotResponse;
pub use rights::Right;
pub use router::{AllowAllPermissions, BotRouter, DenyAllPermissions, PermissionResolver};
pub use trigger::{TriggerAction, TriggerEvent, TriggerHandler};
