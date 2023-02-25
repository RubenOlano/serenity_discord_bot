use std::{collections::HashMap, fs};

use serenity::prelude::*;
use tracing::{debug, warn};

use api::{bot::Bot, schema::circle::Circle};

mod api;
mod commands;
pub mod managers;
mod settings;
mod util;

#[tokio::main]
async fn main() {
    fs::remove_file("./logs/acm-bot.log").unwrap();
    color_eyre::install().unwrap();
    let file = tracing_appender::rolling::never("./logs", "acm-bot.log");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    let sub = tracing_subscriber::fmt().with_writer(non_blocking).finish();

    tracing::subscriber::set_global_default(sub).expect("setting default subscriber failed");

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
        debug!("Starting the cache");
        let mut data = client.data.write().await;
        data.insert::<Circle>(HashMap::new());
    }

    if let Err(why) = client.start().await {
        warn!("Client error: {:#?}", why)
    }
}
