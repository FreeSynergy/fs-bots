// fs-bot/src/rights.rs — Right levels for bot command access control.

/// Access level required to execute a bot command.
///
/// Levels are ordered: `None` < `Member` < `Operator` < `Admin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Right {
    /// Public — no authentication required.
    #[default]
    None,
    /// Authenticated FreeSynergy member.
    Member,
    /// Room operator (can manage subscriptions, sync rules, etc.).
    Operator,
    /// Full administrator.
    Admin,
}
