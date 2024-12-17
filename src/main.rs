use dotenvy::dotenv;
use serenity::prelude::{Client, GatewayIntents};

mod bot;
mod ezgen;

use bot::Bot;

#[tokio::main]
async fn main() {
    // Load environment vars from a .env file
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment
    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN env");
    let ezgen_api_key = std::env::var("EZGEN_TOKEN").expect("Expected EZGEN_TOKEN env");

    // Create a new instance of the Discord client
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot { ezgen_api_key })
        .await
        .expect("Error creating client");

    // Start the client and run the bot.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
