// Gatekeeper commands: /verify, /approve, /deny

use async_trait::async_trait;
use chrono::Utc;
use fsn_bot::{BotCommand, BotResponse, CommandContext, CommandRegistry, Right};
use sqlx::{Row, SqlitePool};

pub fn register_all(registry: &mut CommandRegistry, pool: SqlitePool) {
    registry.register(VerifyCommand  { pool: pool.clone() });
    registry.register(ApproveCommand { pool: pool.clone() });
    registry.register(DenyCommand    { pool });
}

// ── /verify ───────────────────────────────────────────────────────────────────

/// Check a user's IAM status and add a join request if none exists.
pub struct VerifyCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for VerifyCommand {
    fn name(&self) -> &str { "verify" }
    fn description(&self) -> &str { "Verify a user's IAM membership and queue a join request" }
    fn required_right(&self) -> Right { Right::Operator }
    fn usage(&self) -> Option<&str> { Some("verify <user_id>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(user_id) = ctx.arg0() else {
            return BotResponse::error("Usage: /verify <user_id>");
        };
        let platform = ctx.platform.label();
        let room_id  = ctx.room().as_str();

        // Check if request already exists
        let existing = sqlx::query(
            "SELECT id, status FROM join_requests
             WHERE platform = ? AND room_id = ? AND user_id = ?
             ORDER BY id DESC LIMIT 1",
        )
        .bind(platform)
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await;

        match existing {
            Ok(Some(row)) => {
                let id: i64 = row.get(0);
                let status: String = row.get(1);
                BotResponse::text(format!(
                    "Request #{id} for user `{user_id}` already exists (status: {status})."
                ))
            }
            Ok(None) => {
                // TODO Phase P: send iam.check.user Bus event, await response
                // For now: create pending request
                let res = sqlx::query(
                    "INSERT INTO join_requests
                     (platform, room_id, user_id, status, iam_result, created_at)
                     VALUES (?, ?, ?, 'pending', 'iam-check-pending', ?) RETURNING id",
                )
                .bind(platform)
                .bind(room_id)
                .bind(user_id)
                .bind(Utc::now().to_rfc3339())
                .fetch_one(&self.pool)
                .await;

                match res {
                    Ok(row) => {
                        let id: i64 = row.get(0);
                        BotResponse::text(format!(
                            "Join request #{id} queued for `{user_id}`. IAM check pending (Phase P). Use /approve {id} or /deny {id}."
                        ))
                    }
                    Err(e) => BotResponse::error(format!("DB error: {e}")),
                }
            }
            Err(e) => BotResponse::error(format!("DB error: {e}")),
        }
    }
}

// ── /approve ──────────────────────────────────────────────────────────────────

pub struct ApproveCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for ApproveCommand {
    fn name(&self) -> &str { "approve" }
    fn description(&self) -> &str { "Approve a pending join request" }
    fn required_right(&self) -> Right { Right::Operator }
    fn usage(&self) -> Option<&str> { Some("approve <request_id>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(id_str) = ctx.arg0() else {
            return BotResponse::error("Usage: /approve <request_id>");
        };
        let Ok(id) = id_str.parse::<i64>() else {
            return BotResponse::error("Request ID must be a number.");
        };

        match resolve_request(&self.pool, id, "approved").await {
            Ok(user_id) => BotResponse::text(format!(
                "Request #{id} approved. User `{user_id}` can now be invited."
            )),
            Err(e) => BotResponse::error(e),
        }
    }
}

// ── /deny ─────────────────────────────────────────────────────────────────────

pub struct DenyCommand { pub pool: SqlitePool }

#[async_trait]
impl BotCommand for DenyCommand {
    fn name(&self) -> &str { "deny" }
    fn description(&self) -> &str { "Deny a pending join request" }
    fn required_right(&self) -> Right { Right::Operator }
    fn usage(&self) -> Option<&str> { Some("deny <request_id>") }

    async fn execute(&self, ctx: CommandContext) -> BotResponse {
        let Some(id_str) = ctx.arg0() else {
            return BotResponse::error("Usage: /deny <request_id>");
        };
        let Ok(id) = id_str.parse::<i64>() else {
            return BotResponse::error("Request ID must be a number.");
        };

        match resolve_request(&self.pool, id, "denied").await {
            Ok(user_id) => BotResponse::text(format!(
                "Request #{id} denied. User `{user_id}` will not be admitted."
            )),
            Err(e) => BotResponse::error(e),
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

async fn resolve_request(pool: &SqlitePool, id: i64, status: &str) -> Result<String, String> {
    let row = sqlx::query(
        "SELECT user_id, status FROM join_requests WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("DB error: {e}"))?;

    let Some(row) = row else {
        return Err(format!("Request #{id} not found."));
    };

    let current: String = row.get(1);
    if current != "pending" {
        return Err(format!("Request #{id} is already '{current}' — cannot change."));
    }

    let user_id: String = row.get(0);

    sqlx::query(
        "UPDATE join_requests SET status = ?, resolved_at = ? WHERE id = ?",
    )
    .bind(status)
    .bind(Utc::now().to_rfc3339())
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| format!("DB error: {e}"))?;

    Ok(user_id)
}
