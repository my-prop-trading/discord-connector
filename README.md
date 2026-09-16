# discord-connector

Thin HTTP client for the Discord REST API.

Sends a single message to a channel via the existing **OneFunded** bot application
(bot-token transport) and returns the created message id so the caller can build a
permalink. Mirrors `slack-connector`.

- No embeds, attachments, threads or reactions — one plain message per call.
- Holds no credentials or channel state; the caller supplies `bot_token` (once, at
  construction) and `channel_id` (per call). Credentials live in the bridge settings.

```rust
use discord_connector::client::DiscordApiClient;

let client = DiscordApiClient::new(bot_token, None); // None = public Discord API
let msg = client.send_message(&channel_id, "hello").await?;
println!("message id = {}", msg.id);
```

Manual smoke test:

```
DISCORD_BOT_TOKEN=<token> DISCORD_CHANNEL_ID=<id> cargo run --example basic
```
