// TriggerEngine — dispatches Bus events to registered TriggerHandlers.
//
// TriggerHandler, TriggerEvent, and TriggerAction are defined in fs-bot.
// This module holds the runtime engine that collects and relays actions.

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::warn;

pub use fs_bot::trigger::{TriggerAction, TriggerEvent, TriggerHandler};

use crate::audit::AuditLog;

// ── TriggerEngine ─────────────────────────────────────────────────────────────

/// Manages trigger handlers and dispatches Bus events to them.
///
/// Actions returned by handlers are forwarded via the `UnboundedReceiver`
/// returned by [`TriggerEngine::new`] for the runtime to process.
pub struct TriggerEngine {
    handlers: Vec<Arc<dyn TriggerHandler>>,
    audit: AuditLog,
    action_tx: mpsc::UnboundedSender<TriggerAction>,
}

impl TriggerEngine {
    /// Create a new engine.
    ///
    /// Returns the engine and a receiver for [`TriggerAction`]s.
    /// The caller must spawn a task to drain the receiver.
    pub fn new(audit: AuditLog) -> (Self, mpsc::UnboundedReceiver<TriggerAction>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                handlers: Vec::new(),
                audit,
                action_tx: tx,
            },
            rx,
        )
    }

    /// Register a trigger handler (concrete type).
    pub fn register(&mut self, handler: impl TriggerHandler + 'static) {
        self.handlers.push(Arc::new(handler));
    }

    /// Register a boxed trigger handler (used when modules return `Box<dyn TriggerHandler>`).
    pub fn register_boxed(&mut self, handler: Box<dyn TriggerHandler>) {
        self.handlers.push(Arc::from(handler));
    }

    /// Dispatch a Bus event to all matching handlers.
    pub async fn dispatch(&self, event: TriggerEvent) {
        let topic = event.topic.clone();
        for handler in &self.handlers {
            if handler
                .topics()
                .iter()
                .any(|pat| topic_matches(pat, &topic))
            {
                let h = Arc::clone(handler);
                let ev = event.clone();
                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let actions = h.on_event(ev).await;
                    for action in actions {
                        if tx.send(action).is_err() {
                            warn!("TriggerEngine action channel closed");
                        }
                    }
                });
            }
        }
        self.audit
            .system_action(
                &format!("trigger.dispatch:{}", topic),
                None,
                None,
                "ok",
                None,
            )
            .await;
    }

    /// All subscribed topics across all handlers (deduplicated, sorted).
    pub fn subscribed_topics(&self) -> Vec<&str> {
        let mut topics: Vec<&str> = self
            .handlers
            .iter()
            .flat_map(|h| h.topics().iter().copied())
            .collect();
        topics.sort_unstable();
        topics.dedup();
        topics
    }
}

// ── topic_matches ─────────────────────────────────────────────────────────────

/// Glob-style topic matching.
///
/// - `*`  matches exactly one segment
/// - `**` as the last segment matches any remaining suffix
fn topic_matches(pattern: &str, topic: &str) -> bool {
    if pattern == topic || pattern == "**" {
        return true;
    }
    let pat_parts: Vec<&str> = pattern.split('.').collect();
    let top_parts: Vec<&str> = topic.split('.').collect();

    if pat_parts.len() != top_parts.len() {
        if pat_parts.last() == Some(&"**") && top_parts.len() >= pat_parts.len() - 1 {
            return pat_parts[..pat_parts.len() - 1]
                .iter()
                .zip(top_parts.iter())
                .all(|(p, t)| *p == "*" || p == t);
        }
        return false;
    }
    pat_parts
        .iter()
        .zip(top_parts.iter())
        .all(|(p, t)| *p == "*" || p == t)
}

#[cfg(test)]
mod tests {
    use super::topic_matches;

    #[test]
    fn exact_match() {
        assert!(topic_matches("a.b.c", "a.b.c"));
    }
    #[test]
    fn star_segment() {
        assert!(topic_matches("a.*.c", "a.b.c"));
    }
    #[test]
    fn double_star_suffix() {
        assert!(topic_matches("a.**", "a.b.c.d"));
    }
    #[test]
    fn no_match() {
        assert!(!topic_matches("a.b", "a.b.c"));
    }
}
