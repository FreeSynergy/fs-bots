# CLAUDE.md – fs-bots

## What is this?

FreeSynergy Bot Runtime — workspace with the bot command framework, domain models,
database layer, and platform-specific bot modules.

## Rules

- Language in files: **English** (comments, code, variable names)
- Language in chat: **German**
- OOP everywhere: traits over match blocks, types carry their own behavior
- No CHANGELOG.md
- After every feature: commit directly

## Quality Gates (before every commit)

```
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test
```

Every lib.rs / main.rs must have:
```rust
#![deny(clippy::all, clippy::pedantic, warnings)]
```

## Workspace

| Crate | Description |
|---|---|
| `fs-bot` | Bot command framework (BotCommand trait, CommandRegistry, BotResponse) |
| `bot-db` | Database layer (SeaORM + SQLite via fs-db) |
| `bot-broadcast` | Broadcast module — scheduled announcements |
| `bot-gatekeeper` | Gatekeeper module — access control and onboarding |
| `bot-calendar` | Calendar module — event notifications |
| `bot-control` | Control module — admin commands |
| `bot-room-sync` | Room-sync module — cross-platform room bridging |
| `bots` | Runtime binary (`fs-bot-runtime`) — event loop, dispatcher, trigger engine |

## Dependencies

- `fs-channel` — messaging channel abstraction (Telegram, Matrix)
- `fs-db` — SQLite database via SeaORM
- `fs-bus` — internal message bus
- `fs-types` — shared FreeSynergy types (MessengerKind, etc.)
- `tokio` — async runtime
- `axum` — webhook HTTP server
