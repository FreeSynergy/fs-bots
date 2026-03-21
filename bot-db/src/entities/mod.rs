// Bot domain entities — SeaORM models for all bot manager tables.
//
// Each sub-module is one table. The BotDb struct in db.rs uses these
// entities via typed repository methods — no raw SQL anywhere else.

pub mod audit_log;
pub mod bot_meta;
pub mod child_bot;
pub mod join_request;
pub mod known_room;
pub mod poll_state;
pub mod room_collection;
pub mod room_collection_member;
pub mod subscription;
pub mod sync_message;
pub mod sync_rule;
