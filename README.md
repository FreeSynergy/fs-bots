# fs-bots

Bot runtime workspace for FreeSynergy — command framework, database layer,
platform modules, and the runtime binary.

## Build

```sh
cargo build --release
cargo test
```

## Workspace

| Crate | Description |
|---|---|
| `fs-bot` | Bot command framework (BotCommand trait, registry, response) |
| `bot-db` | Database domain objects and SeaORM entities |
| `bot-broadcast` | Scheduled broadcast module |
| `bot-gatekeeper` | Access control and onboarding module |
| `bot-calendar` | Calendar event notification module |
| `bot-control` | Admin control module |
| `bot-room-sync` | Cross-platform room sync module |
| `bots` | Runtime binary (`fs-bot-runtime`) |

## Architecture

- `BotCommand` trait — each command is an object implementing name/description/execute
- `CommandRegistry` — dispatches incoming `/commands` to the right handler
- `BotRuntime` — core event loop: polls messengers, handles webhooks, routes triggers
- `CommandDispatcher` — bridges IncomingMessage to CommandRegistry with rights check
- `TriggerEngine` — evaluates time/event triggers and routes `TriggerAction`s
- `BotDb` — SQLite-backed persistence (subscriptions, audit log, offsets)
