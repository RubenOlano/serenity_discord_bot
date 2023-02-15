mod api;
mod commands;
pub mod managers;
mod settings;

use std::collections::HashMap;

use api::{bot::Bot, schema::circle::Circle};
use serenity::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let bot = Bot::new().await;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::GUILD_EMOJIS_AND_STICKERS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let mut client = Client::builder(bot.settings.token.clone(), intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<Circle>(HashMap::new());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {why:#?}");
    }
}
