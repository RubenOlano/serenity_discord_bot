use color_eyre::Result;
use eyre::{Context, ContextCompat};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

use crate::api::bot::Bot;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("admin")
        .description("Look up a user's information")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to look up")
                .kind(CommandOptionType::User)
                .required(true)
        })
}

pub async fn run(options: &[CommandDataOption], bot: &Bot) -> Result<String> {
    let data = options
        .get(0)
        .expect("No options provided")
        .resolved
        .as_ref()
        .context("No user provided")?;

    let user_id = match data {
        CommandDataOptionValue::User(user, _member) => user.id,
        _ => panic!("No user provided"),
    };

    let name: Option<String> = bot
        .firestore_manager
        .client
        .fluent()
        .select()
        .by_id_in("discord")
        .obj()
        .one(user_id.to_string())
        .await
        .context("Failed to get user")?;

    match name {
        Some(name) => Ok(format!("{} is {}", user_id, name)),
        None => Ok(format!("{} is not in the database", user_id)),
    }
}
