// bot-calendar — N8: Calendar reminders via Bus + /termine command.
//
// Commands: /termine
// Trigger:  CalendarHandler listens on "calendar.event.*" and sends reminders.

use fsn_bot::{CommandRegistry, TriggerHandler};

mod commands;
mod trigger;

/// Register calendar commands and return the trigger handler.
pub fn register(registry: &mut CommandRegistry) -> Vec<Box<dyn TriggerHandler>> {
    commands::register_all(registry);
    vec![Box::new(trigger::CalendarHandler)]
}
