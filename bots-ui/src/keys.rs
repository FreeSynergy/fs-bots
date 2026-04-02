// keys.rs — FTL key name constants for fs-bots.
//
// All user-visible strings are translated via fs-i18n.
// The matching .ftl files live at:
//   fs-i18n/locales/{lang}/bots.ftl
//
// Use these constants wherever a localised string is needed.

// ── App ───────────────────────────────────────────────────────────────────────

pub const TITLE: &str = "bots-title";
pub const HOME: &str = "bots-home";

// ── Status ────────────────────────────────────────────────────────────────────

pub const STATUS_ENABLED: &str = "bots-status-enabled";
pub const STATUS_DISABLED: &str = "bots-status-disabled";

// ── Control bot ───────────────────────────────────────────────────────────────

pub const CONTROL_TITLE: &str = "bots-control-title";
pub const CONTROL_DESCRIPTION: &str = "bots-control-description";

// ── Accounts ──────────────────────────────────────────────────────────────────

pub const ACCOUNTS_TITLE: &str = "bots-accounts-title";
pub const ACCOUNTS_EMPTY: &str = "bots-accounts-empty";
pub const ACCOUNTS_ADD: &str = "bots-accounts-add";
pub const ACCOUNTS_CONNECTED: &str = "bots-accounts-connected";
pub const ACCOUNTS_DISCONNECTED: &str = "bots-accounts-disconnected";

// ── Messengers ────────────────────────────────────────────────────────────────

pub const MESSENGER_MATRIX: &str = "bots-messenger-matrix";
pub const MESSENGER_MATTERMOST: &str = "bots-messenger-mattermost";
pub const MESSENGER_DISCORD: &str = "bots-messenger-discord";

// ── Groups ────────────────────────────────────────────────────────────────────

pub const GROUPS_TITLE: &str = "bots-groups-title";
pub const GROUP_CREATE: &str = "bots-group-create";

// ── Broadcast ─────────────────────────────────────────────────────────────────

pub const BROADCAST_TITLE: &str = "bots-broadcast-title";
pub const BROADCAST_TARGET_EMPTY: &str = "bots-broadcast-target-empty";
pub const BROADCAST_MESSAGE_EMPTY: &str = "bots-broadcast-message-empty";

// ── Gatekeeper ────────────────────────────────────────────────────────────────

pub const GATEKEEPER_TITLE: &str = "bots-gatekeeper-title";
pub const GATEKEEPER_PENDING: &str = "bots-gatekeeper-pending";

// ── Bot types ─────────────────────────────────────────────────────────────────

pub const TYPE_BROADCAST: &str = "bots-type-broadcast";
pub const TYPE_DIGEST: &str = "bots-type-digest";
pub const TYPE_GATEKEEPER: &str = "bots-type-gatekeeper";
pub const TYPE_MONITOR: &str = "bots-type-monitor";
