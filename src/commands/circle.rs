use color_eyre::Result;
use mongodb::bson::{doc, DateTime, Document};
use serenity::builder::CreateApplicationCommand;
use serenity::model::channel::PermissionOverwriteType;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::{ChannelId, GuildId, PermissionOverwrite, Role, RoleId};
use serenity::model::Permissions;
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
    .create_option(|option| {
        option
            .name("repost")
            .description("Repost the circle embeds")
            .kind(CommandOptionType::SubCommand)
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
        "repost" => repost(&subcommand.options, ctx, bot).await?,
        _ => return Err(eyre::eyre!("Invalid subcommand provided")),
    };

    Ok(res)
}

pub async fn repost(_: &[CommandDataOption], ctx: &Context, bot: &Bot) -> Result<String> {
    bot.circle_manager.repost(ctx).await?;
    Ok("Done!".to_string())
}

pub async fn add(options: &[CommandDataOption], ctx: &Context, bot: &Bot) -> Result<String> {
    let role = create_role(options, ctx, bot).await?;
    add_role_to_owner(options, ctx, bot, &role).await?;
    let channel = create_channel(options, ctx, bot, role.id).await?;

    let circle_data = parse_circle_add_options(options, channel, role.id)?;
    bot.mongo_manager.circle_add(ctx, circle_data).await?;

    Ok("Circle added".to_string())
}

fn test_emoji(emoji: &str) -> bool {
    let test_reg = regex::Regex::new(r"\p{Extended_Pictographic}");
    let Ok(test_reg) = test_reg else { return false; };
    info!("Testing emoji: {}", emoji);
    test_reg.is_match(emoji)
}

fn parse_circle_add_options(
    options: &[CommandDataOption],
    channel: ChannelId,
    role: RoleId,
) -> Result<Document> {
    let name = parse_option(options, "name")?;
    let name = match name {
        CommandDataOptionValue::String(name) => name,
        _ => Err(eyre::eyre!("No name provided"))?,
    };

    let description = parse_option(options, "description")?;
    let description = match description {
        CommandDataOptionValue::String(description) => description,
        _ => Err(eyre::eyre!("No description provided"))?,
    };

    let emoji = parse_option(options, "emoji")?;
    let emoji = match emoji {
        CommandDataOptionValue::String(emoji) => emoji,
        _ => Err(eyre::eyre!("No emoji provided"))?,
    };

    let graphic = parse_option(options, "graphic")?;
    let graphic = match graphic {
        CommandDataOptionValue::String(graphic) => graphic,
        _ => Err(eyre::eyre!("No graphic provided"))?,
    };

    let owner = parse_option(options, "owner")?;
    let owner = match owner {
        CommandDataOptionValue::User(owner, _member) => owner,
        _ => Err(eyre::eyre!("No owner provided"))?,
    };

    if !test_emoji(emoji) {
        return Err(eyre::eyre!("Invalid emoji"));
    }

    let circle = doc! {
        "name": name.to_string(),
        "description": description.to_string(),
        "emoji": emoji.to_string(),
        "imageUrl": graphic.to_string(),
        "owner": owner.id.to_string(),
        "channel": channel.to_string(),
        "createdOn": DateTime::now(),
        "subChannels": Vec::<String>::new(),
        "_id": role.to_string(),
    };

    Ok(circle)
}

async fn create_role(options: &[CommandDataOption], ctx: &Context, bot: &Bot) -> Result<Role> {
    let name = parse_option(options, "name")?;
    let name = match name {
        CommandDataOptionValue::String(name) => name,
        _ => Err(eyre::eyre!("No name provided"))?,
    };

    let color = parse_option(options, "color")?;
    let color = match color {
        CommandDataOptionValue::String(color) => color,
        _ => Err(eyre::eyre!("No color provided"))?,
    };
    let color_int = color
        .parse::<u64>()
        .map_err(|_| eyre::eyre!("Invalid color: {}", color))?;

    let emoji = parse_option(options, "emoji")?;
    let emoji = match emoji {
        CommandDataOptionValue::String(emoji) => emoji,
        _ => Err(eyre::eyre!("No emoji provided"))?,
    };

    let guild_id = GuildId(bot.settings.guild);
    let res = guild_id
        .create_role(&ctx.http, |r| {
            r.name(format!("{emoji} {name}"))
                .colour(color_int)
                .mentionable(true)
        })
        .await?;

    Ok(res)
}

async fn add_role_to_owner(
    options: &[CommandDataOption],
    ctx: &Context,
    bot: &Bot,
    role: &Role,
) -> Result<()> {
    let owner = parse_option(options, "owner")?;
    let owner = match owner {
        CommandDataOptionValue::User(owner, _member) => owner,
        _ => Err(eyre::eyre!("No owner provided"))?,
    };

    let guild_id = GuildId(bot.settings.guild);

    let mut member = guild_id.member(&ctx.http, owner.id).await?;
    member.add_role(&ctx.http, role.id).await?;

    Ok(())
}

async fn create_channel(
    options: &[CommandDataOption],
    ctx: &Context,
    bot: &Bot,
    role: RoleId,
) -> Result<ChannelId> {
    let name = parse_option(options, "name")?;
    let name = match name {
        CommandDataOptionValue::String(name) => name,
        _ => Err(eyre::eyre!("No name provided"))?,
    };

    let description = parse_option(options, "description")?;
    let description = match description {
        CommandDataOptionValue::String(description) => description,
        _ => Err(eyre::eyre!("No description provided"))?,
    };

    let emoji = parse_option(options, "emoji")?;
    let emoji = match emoji {
        CommandDataOptionValue::String(emoji) => emoji,
        _ => Err(eyre::eyre!("No emoji provided"))?,
    };

    let guild_id = GuildId(bot.settings.guild);

    let roles = guild_id.roles(&ctx.http).await?;
    let everyone = roles
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
                        kind: PermissionOverwriteType::Role(role),
                    },
                    PermissionOverwrite {
                        allow: Permissions::empty(),
                        deny: Permissions::VIEW_CHANNEL,
                        kind: PermissionOverwriteType::Role(*everyone.0),
                    },
                ])
        })
        .await?;

    Ok(res.id)
}

fn parse_option<'a>(
    options: &'a [CommandDataOption],
    name: &'a str,
) -> Result<&'a CommandDataOptionValue> {
    let option = options
        .iter()
        .find(|o| o.name == name)
        .ok_or(eyre::eyre!("No {} provided", name));

    let option = match option {
        Ok(option) => option,
        Err(e) => return Err(e),
    };

    let resolved = option
        .resolved
        .as_ref()
        .ok_or(eyre::eyre!("No {} provided", name))?;

    Ok(resolved)
}
