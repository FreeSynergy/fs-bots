// Gatekeeper commands: /verify, /approve, /deny

use async_trait::async_trait;
use bot_db::BotDb;
use fs_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use std::sync::Arc;

pub fn register_all(registry: &mut CommandRegistry, db: Arc<BotDb>) {
    registry.register(VerifyCommand { db: db.clone() });
    registry.register(ApproveCommand { db: db.clone() });
    registry.register(DenyCommand { db });
}

pub struct VerifyCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for VerifyCommand {
    fn name(&self) -> &'static str {
        "verify"
    }
    fn description(&self) -> &'static str {
        "Verify a user's IAM membership and queue a join request"
    }
    fn required_right(&self) -> Right {
        Right::Operator
    }
    fn usage(&self) -> Option<&str> {
        Some("verify <user_id>")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(user_id) = ctx.arg0() else {
            return BotResponse::error("Usage: /verify <user_id>");
        };
        let platform = ctx.platform.as_str();
        let room_id = ctx.room_id.as_str();

        // Check if request already exists
        match self.db.list_pending_join_requests(platform, room_id).await {
            Err(e) => BotResponse::error(format!("DB error: {e}")),
            Ok(existing) => {
                if let Some(req) = existing.iter().find(|r| r.user_id == user_id) {
                    return BotResponse::text(format!(
                        "Request #{} for user `{user_id}` already exists (status: {}).",
                        req.id, req.status
                    ));
                }
                // TODO Phase P: send iam.check.user Bus event, await response
                match self.db.add_join_request(platform, room_id, user_id).await {
                    Ok(id) => BotResponse::text(format!(
                        "Join request #{id} queued for `{user_id}`. IAM check pending (Phase P). Use /approve {id} or /deny {id}."
                    )),
                    Err(e) => BotResponse::error(format!("DB error: {e}")),
                }
            }
        }
    }
}

pub struct ApproveCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for ApproveCommand {
    fn name(&self) -> &'static str {
        "approve"
    }
    fn description(&self) -> &'static str {
        "Approve a pending join request"
    }
    fn required_right(&self) -> Right {
        Right::Operator
    }
    fn usage(&self) -> Option<&str> {
        Some("approve <request_id>")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(id_str) = ctx.arg0() else {
            return BotResponse::error("Usage: /approve <request_id>");
        };
        let Ok(id) = id_str.parse::<i64>() else {
            return BotResponse::error("Request ID must be a number.");
        };
        match self.db.resolve_join_request(id, "approved", None).await {
            Ok(()) => {
                let user = self
                    .db
                    .get_join_request(id)
                    .await
                    .ok()
                    .flatten()
                    .map_or_else(|| "unknown".to_string(), |r| r.user_id);
                BotResponse::text(format!(
                    "Request #{id} approved. User `{user}` can now be invited."
                ))
            }
            Err(e) => BotResponse::error(format!("{e}")),
        }
    }
}

pub struct DenyCommand {
    pub db: Arc<BotDb>,
}

#[async_trait]
impl BotCommand for DenyCommand {
    fn name(&self) -> &'static str {
        "deny"
    }
    fn description(&self) -> &'static str {
        "Deny a pending join request"
    }
    fn required_right(&self) -> Right {
        Right::Operator
    }
    fn usage(&self) -> Option<&str> {
        Some("deny <request_id>")
    }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(id_str) = ctx.arg0() else {
            return BotResponse::error("Usage: /deny <request_id>");
        };
        let Ok(id) = id_str.parse::<i64>() else {
            return BotResponse::error("Request ID must be a number.");
        };
        match self.db.resolve_join_request(id, "denied", None).await {
            Ok(()) => BotResponse::text(format!("Request #{id} denied.")),
            Err(e) => BotResponse::error(format!("{e}")),
        }
    }
}
