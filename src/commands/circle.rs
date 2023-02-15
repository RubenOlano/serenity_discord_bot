use eyre::ContextCompat;
use mongodb::bson::{doc, DateTime};
use serenity::builder::CreateApplicationCommand;
use serenity::model::channel::PermissionOverwriteType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::{GuildId, PermissionOverwrite};
use serenity::model::Permissions;

use color_eyre::Result;
use serenity::prelude::Context;
use tracing::info;

use crate::api::bot::Bot;
pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.create_option(|option| {
        option
            .name("add")
            .description("Add a new circle")
            .kind(CommandOptionType::SubCommand)
            .create_sub_option(|option| {
                option
                    .name("name")
                    .description("The name of the circle")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_sub_option(|option| {
                option
                    .name("description")
                    .description("The description of the circle")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_sub_option(|o| {
                o.name("color")
                    .description("The color of the circle")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_sub_option(|o| {
                o.name("emoji")
                    .description("The emoji of the circle")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_sub_option(|o| {
                o.name("graphic")
                    .description("The graphic of the circle")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_sub_option(|o| {
                o.name("owner")
                    .description("The owner of the circle")
                    .kind(CommandOptionType::User)
                    .required(true)
            })
    })
    .name("circle")
    .description("Manage circles")
}

pub async fn run(options: &[CommandDataOption], ctx: &Context, bot: &Bot) -> Result<String> {
    info!("Running circle command");
    let subcommand = options
        .get(0)
        .ok_or(eyre::eyre!("No subcommand provided"))?;

    let sub_cmd_name = subcommand.name.as_str();

    let res = match sub_cmd_name {
        "add" => add(&subcommand.options, ctx, bot).await?,
        _ => panic!("Unknown subcommand"),
    };

    Ok(res)
}

pub async fn add(options: &[CommandDataOption], ctx: &Context, bot: &Bot) -> Result<String> {
    let name = options
        .get(0)
        .ok_or(eyre::eyre!("No options provided"))?
        .resolved
        .as_ref()
        .context("No name provided")?;

    let name = match name {
        CommandDataOptionValue::String(name) => name,
        _ => Err(eyre::eyre!("No name provided"))?,
    };

    let description = options
        .get(1)
        .ok_or(eyre::eyre!("No options provided"))?
        .resolved
        .as_ref()
        .context("No description provided")?;

    let description = match description {
        CommandDataOptionValue::String(description) => description,
        _ => Err(eyre::eyre!("No description provided"))?,
    };

    let color = options
        .get(2)
        .ok_or(eyre::eyre!("No options provided"))?
        .resolved
        .as_ref()
        .context("No color provided")?;

    let color = match color {
        CommandDataOptionValue::String(color) => color,
        _ => Err(eyre::eyre!("No color provided"))?,
    };

    let emoji = options
        .get(3)
        .ok_or(eyre::eyre!("No options provided"))?
        .resolved
        .as_ref()
        .context("No emoji provided")?;

    let emoji = match emoji {
        CommandDataOptionValue::String(emoji) => emoji,
        _ => Err(eyre::eyre!("No emoji provided"))?,
    };

    let graphic = options
        .get(4)
        .ok_or(eyre::eyre!("No options provided"))?
        .resolved
        .as_ref()
        .context("No graphic provided")?;

    let graphic = match graphic {
        CommandDataOptionValue::String(graphic) => graphic,
        _ => Err(eyre::eyre!("No graphic provided"))?,
    };

    let owner = options
        .get(5)
        .ok_or(eyre::eyre!("No options provided"))?
        .resolved
        .as_ref()
        .context("No owner provided")?;

    let owner = match owner {
        CommandDataOptionValue::User(owner, _member) => owner,
        _ => Err(eyre::eyre!("No owner provided"))?,
    };

    if !test_emoji(emoji) {
        return Err(eyre::eyre!("Invalid emoji"));
    }
    let color_int = color
        .parse::<u64>()
        .map_err(|_| eyre::eyre!("Invalid color"))?;

    let guild_id = GuildId(bot.settings.guild);
    let res = guild_id
        .create_role(&ctx.http, |r| {
            r.name(format!("{emoji} {name}"))
                .colour(color_int)
                .mentionable(true)
        })
        .await?;

    let mut member = guild_id.member(&ctx.http, owner.id).await?;
    member.add_role(&ctx.http, res.id).await?;

    let everyone = guild_id.roles(&ctx.http).await?;
    let everyone = everyone
        .iter()
        .find(|(_id, role)| role.name == "@everyone")
        .ok_or(eyre::eyre!("No @everyone role found"))?;

    let res = guild_id
        .create_channel(&ctx.http, |c| {
            c.name(format!("{emoji} {name}"))
                .kind(serenity::model::channel::ChannelType::Text)
                .category(bot.settings.circles.parent_category)
                .topic(description)
                .permissions(vec![
                    PermissionOverwrite {
                        allow: Permissions::VIEW_CHANNEL,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Role(res.id),
                    },
                    PermissionOverwrite {
                        allow: Permissions::empty(),
                        deny: Permissions::VIEW_CHANNEL,
                        kind: PermissionOverwriteType::Role(*everyone.0),
                    },
                ])
        })
        .await?;

    let circle = doc! {
        "name": name,
        "description": description,
        "emoji": emoji,
        "imageUrl": graphic,
        "owner": owner.id.to_string(),
        "createdOn": DateTime::now(),
        "channel": res.id.to_string(),
        "subChannels": [],
    };

    bot.mongo_manager.circle_add(ctx, circle).await?;

    Ok("Circle added".to_string())
}

fn test_emoji(emoji: &str) -> bool {
    let test_reg = regex::Regex::new(r"\p{Extended_Pictographic}");
    let test_reg = match test_reg {
        Ok(reg) => reg,
        Err(_) => return false,
    };
    info!("Testing emoji: {}", emoji);
    test_reg.is_match(emoji)
}
