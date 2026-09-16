use serde::{Deserialize, Serialize};

/// Body of `POST /channels/{channel_id}/messages`.
/// Only `content` is used — no embeds, attachments, threads or reactions (spec §4:
/// "one message per payout, without threads and reactions", "no attachments").
#[derive(Serialize, Debug)]
pub struct CreateMessageRequest {
    pub content: String,
}

/// Subset of Discord's message object we need back: the created message id, used to
/// build the permalink stored on the payout row.
#[derive(Deserialize, Debug)]
pub struct MessageResponse {
    pub id: String,
    /// Echoed back by Discord; kept for permalink assembly / diagnostics.
    #[serde(default)]
    pub channel_id: Option<String>,
}

/// Discord error payload (non-2xx), e.g. `{"message":"Missing Access","code":50001}`.
#[derive(Deserialize, Debug)]
pub struct DiscordErrorResponse {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub code: Option<i64>,
}
