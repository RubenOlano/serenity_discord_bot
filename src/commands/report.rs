use color_eyre::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::InteractionResponseType;

use crate::api::bot::Bot;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    cmd.name("Anonymous Report")
        .kind(CommandType::Message)
}

pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction, bot: &Bot) -> Result<String> {
    let data = cmd.data.resolved.clone();
    let msg: Vec<_> = data.messages.values().collect();
    let Some(msg) = msg.first() else {
        return Err(eyre::eyre!("Unable to get message"));
    };
    let (embed, action_row) = bot.report_manager.handle_init_report(msg, ctx, cmd).await?;

    cmd.create_interaction_response(&ctx.http, |res| {
        res.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message
                    .content("Please select a category for your report")
                    .components(|c| c.add_action_row(action_row))
                    .set_embed(embed)
                    .ephemeral(true)
            })
    }).await?;

    Ok("Thank you for your report".to_string())
}