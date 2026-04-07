// Bot domain entities — plain domain structs for all bot manager tables.
//
// No SeaORM macros — persistence goes through DbEngine (fs-db abstraction).
// Each sub-module is one table; each struct has a `from_row` method that
// maps a DbRow to the struct.

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
