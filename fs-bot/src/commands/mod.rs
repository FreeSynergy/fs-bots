// fs-bot/src/commands/mod.rs — Standard built-in commands.

pub mod deploy;
pub mod health;
pub mod help;
pub mod ping;
pub mod status;

pub use deploy::DeployCommand;
pub use health::{HealthQueryCommand, HealthQueryProvider, StubHealthProvider};
pub use help::HelpCommand;
pub use ping::PingCommand;
pub use status::{DefaultStatusProvider, StatusCommand, StatusProvider};
