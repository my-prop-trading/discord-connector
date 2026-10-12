use serde::{Deserialize, Serialize};

/// Body of `POST /channels/{channel_id}/messages`.
/// Only `content` is used — no embeds, attachments, threads or reactions (spec §4:
/// "one message per payout, without threads and reactions", "no attachments").
///
/// `nonce` + `enforce_nonce` make the send idempotent: if the same author already
/// created a message with this nonce in the past few minutes, Discord returns that
/// message instead of creating a new one. Max 25 characters.
#[derive(Serialize, Debug)]
pub struct CreateMessageRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_nonce: Option<bool>,
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

/// A failed send. `http_status` is Discord's HTTP status when a response arrived (so a
/// caller can tell "no access" 403 / "bad token" 401 / "unknown channel" 404 from a
/// rate limit or a server error without parsing text), None when none did (network,
/// TLS, timeout). `message` is the full human-readable text.
#[derive(Debug, Clone)]
pub struct DiscordSendError {
    pub http_status: Option<u16>,
    pub message: String,
}

impl std::fmt::Display for DiscordSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Discord error payload (non-2xx), e.g. `{"message":"Missing Access","code":50001}`.
#[derive(Deserialize, Debug)]
pub struct DiscordErrorResponse {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub code: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_fields_are_omitted_when_absent() {
        let body = CreateMessageRequest { content: "hi".into(), nonce: None, enforce_nonce: None };
        assert_eq!(serde_json::to_string(&body).unwrap(), r#"{"content":"hi"}"#);
    }

    #[test]
    fn nonce_is_enforced_when_present() {
        let body = CreateMessageRequest {
            content: "hi".into(),
            nonce: Some("lp0123456789abcdef".into()),
            enforce_nonce: Some(true),
        };
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            r#"{"content":"hi","nonce":"lp0123456789abcdef","enforce_nonce":true}"#
        );
    }
}
