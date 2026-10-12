use flurl::body::FlUrlBody;
use flurl::FlUrl;

use crate::model::{CreateMessageRequest, DiscordErrorResponse, DiscordSendError, MessageResponse};

/// Default Discord REST API base. Overridable so tests / non-prod can point elsewhere.
pub const DEFAULT_API_BASE: &str = "https://discord.com/api/v10";

/// Thin Discord REST client using the bot-token transport of the existing OneFunded
/// application. Holds no channel state — the caller passes the channel per call, so one
/// client can serve multiple channels. Credentials live in bridge settings, not here.
pub struct DiscordApiClient {
    api_base: String,
    bot_token: String,
}

impl DiscordApiClient {
    /// `bot_token` is the raw token of the OneFunded bot application (no "Bot " prefix —
    /// added here). `api_base` defaults to the public Discord API when `None`.
    /// A provided base MUST include the scheme (http/https): without it flurl can't
    /// resolve the host and treats the first path segment (the channel id) as the
    /// host, connecting to "<channel_id>:80" — so a schemeless/blank base is ignored
    /// and the default is used instead.
    pub fn new(bot_token: String, api_base: Option<String>) -> Self {
        let api_base = api_base
            .filter(|s| s.starts_with("http://") || s.starts_with("https://"))
            .unwrap_or_else(|| DEFAULT_API_BASE.to_string());
        Self { api_base, bot_token }
    }

    /// Post one message to a channel. Returns the created message id on success.
    /// `POST {api_base}/channels/{channel_id}/messages` with `Authorization: Bot <token>`.
    pub async fn send_message(
        &self,
        channel_id: &str,
        content: &str,
    ) -> Result<MessageResponse, String> {
        self.send_message_with_nonce(channel_id, content, None)
            .await
            .map_err(|e| e.message)
    }

    /// Like `send_message`, but idempotent when `nonce` is given: a repeat of the same
    /// request (retry after a timeout, a resent gRPC call) within Discord's nonce window
    /// (a few minutes) returns the message created by the first one instead of posting
    /// a duplicate. The nonce must be at most 25 characters and stable per message.
    pub async fn send_message_with_nonce(
        &self,
        channel_id: &str,
        content: &str,
        nonce: Option<&str>,
    ) -> Result<MessageResponse, DiscordSendError> {
        let no_response = |message: String| DiscordSendError { http_status: None, message };
        let nonce = nonce.filter(|n| !n.is_empty());
        let payload = CreateMessageRequest {
            content: content.to_string(),
            nonce: nonce.map(|n| n.to_string()),
            enforce_nonce: nonce.map(|_| true),
        };

        let response = FlUrl::new(self.api_base.clone())
            .append_path_segment("channels")
            .append_path_segment(channel_id)
            .append_path_segment("messages")
            .with_header("Content-Type", "application/json")
            .with_header("Authorization", format!("Bot {}", self.bot_token))
            // Discord requires a User-Agent on all API requests.
            .with_header("User-Agent", "OneFunded-live-payouts (discord-connector, 0.1.3)")
            .post(FlUrlBody::as_json(&payload))
            .await
            .map_err(|e| no_response(format!("Discord HTTP POST failed: {:?}", e)))?;

        let status_code = response.get_status_code();
        let with_status = |message: String| DiscordSendError {
            http_status: Some(status_code as u16),
            message,
        };
        let body = response
            .receive_body()
            .await
            .map_err(|e| with_status(format!("Failed to receive Discord body: {:?}", e)))?;
        let body_str = String::from_utf8(body)
            .map_err(|e| with_status(format!("Discord body not UTF-8: {}", e)))?;

        // Discord returns 200/201 on a created message.
        if (200..300).contains(&status_code) {
            let parsed: MessageResponse = serde_json::from_str(&body_str).map_err(|e| {
                with_status(format!(
                    "Failed to parse Discord message response: {:?}. Body: {}",
                    e, body_str
                ))
            })?;
            return Ok(parsed);
        }

        // Surface Discord's own error message so the row's last_error is actionable
        // (spec §4: "the full error text, not just 'error'").
        let detail = match serde_json::from_str::<DiscordErrorResponse>(&body_str) {
            Ok(err) => format!(
                "{} (code {})",
                err.message.unwrap_or_else(|| "unknown".to_string()),
                err.code.map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
            ),
            Err(_) => body_str.clone(),
        };
        Err(with_status(format!(
            "Discord send_message failed: HTTP {} channel {} — {}",
            status_code, channel_id, detail
        )))
    }
}
