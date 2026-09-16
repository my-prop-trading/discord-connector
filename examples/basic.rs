use discord_connector::client::DiscordApiClient;

// Manual smoke test against a real (test) channel:
//   DISCORD_BOT_TOKEN=<token> DISCORD_CHANNEL_ID=<id> cargo run --example basic
#[tokio::main]
async fn main() {
    // A bare binary must install a rustls crypto provider before the first TLS
    // connect; the bridge gets this from service-sdk instead.
    my_tls::install_default_crypto_providers();

    let token = std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN not set");
    let channel_id = std::env::var("DISCORD_CHANNEL_ID").expect("DISCORD_CHANNEL_ID not set");

    let client = DiscordApiClient::new(token, None);
    let content = "🎉 Chukwuemeka from 🇳🇬 just secured a $1,021.97 reward! 💰 2026-08-19";

    match client.send_message(&channel_id, content).await {
        Ok(msg) => println!("sent: message id = {}", msg.id),
        Err(e) => println!("failed: {}", e),
    }
    println!("Run basic example: FINISHED");
}
