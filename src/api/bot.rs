use color_eyre::Report;
use color_eyre::Result;
use serenity::model::application::interaction::message_component::MessageComponentInteraction;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::{
    async_trait,
    model::prelude::{
        component::ComponentType,
        interaction::{Interaction, InteractionResponseType},
        Activity, GuildId, Ready,
    },
    prelude::{Context, EventHandler},
};
use tracing::{info, warn};

use crate::{
    api::schema::circle::Circle,
    commands,
    managers::{circle::CircleManager, mongo::Mongo},
};

// use super::super::managers::firestore::FirestoreManager;
use super::super::settings::Settings;

pub struct Bot {
    pub settings: Settings,
    // pub firestore_manager: FirestoreManager,
    pub mongo_manager: Mongo,
    pub circle_manager: CircleManager,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::watching(&self.settings.activity.description))
            .await;

        // Register commands
        let guild_id = GuildId(self.settings.guild);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                // .create_application_command(|cmd| commands::admin::register(cmd))
                .create_application_command(|cmd| commands::circle::register(cmd))
                .create_application_command(|cmd| {
                    cmd.name("recache").description("Recache the bot")
                })
                .create_application_command(|cmd| commands::ping::register(cmd))
        })
        .await
        .unwrap_or_else(|why| {
            warn!("Cannot register commands: {:?}", why);
            Vec::new()
        });

        for command in commands {
            info!("Registered command: {:?}", command.name);
        }

        self.recache_ctx(&ctx).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(cmd) => {
                if let Err(why) = self.handle_slash(&ctx, cmd).await {
                    warn!("Error handling slash command: {:?}", why);
                }
            }
            Interaction::MessageComponent(component) => {
                if let Err(why) = self.handle_button(&ctx, component).await {
                    warn!("Error handling component: {:?}", why);
                }
            }
            _ => {}
        }
    }
}

impl Bot {
    pub async fn new() -> Self {
        let settings = Settings::new();
        // let firestore_manager = FirestoreManager::new().await;
        let mongo_manager = Mongo::new(&settings).await;
        let circle_manager = CircleManager::new(&settings);
        Self {
            settings,
            // firestore_manager,
            mongo_manager,
            circle_manager,
        }
    }

    pub async fn recache_ctx(&self, ctx: &Context) -> Result<String> {
        let Ok(mut cursor) = self
            .mongo_manager
            .client
            .database("discord")
            .collection::<Circle>("circle")
            .find(None, None)
            .await else {
            warn!("Unable to get circles from database");
            return Ok("Unable to get circles from database".to_string());
        };

        let mut data = ctx.data.write().await;
        let Some(c_manager) = data.get_mut::<Circle>() else {
            warn!("Unable to get circle manager");
            return Ok("Unable to get circle manager".to_string());
        };

        let new_circles = super::super::util::fetch_circles(&mut cursor).await?;
        for (_, circle) in new_circles {
            c_manager.insert(circle.id.clone(), circle);
        }
        Ok("Recached".to_string())
    }

    async fn handle_slash(&self, ctx: &Context, cmd: ApplicationCommandInteraction) -> Result<()> {
        info!("Command: {:?}", cmd.data.name);

        let content = match cmd.data.name.as_str() {
            // "admin" => commands::admin::run(&cmd.data.options, self).await,
            "circle" => commands::circle::run(&cmd.data.options, ctx, self).await,
            "recache" => self.recache_ctx(ctx).await,
            "beep" => Ok(commands::ping::run()),
            _ => Err(Report::msg("Unknown command")),
        };

        match content {
            Ok(content) => {
                let res = cmd
                    .create_interaction_response(&ctx.http, |res| {
                        res.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(content).ephemeral(true)
                            })
                    })
                    .await;
                if let Err(why) = res {
                    warn!("Cannot respond to command: {:?}", why);
                }
            }
            Err(why) => {
                warn!("Cannot respond to command: {:?}", why);
                let res = cmd
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(why.to_string()).ephemeral(true)
                            })
                    })
                    .await;
                if let Err(why) = res {
                    warn!("Cannot respond to command: {:?}", why);
                }
            }
        }
        Ok(())
    }
    async fn handle_button(&self, ctx: &Context, msg: MessageComponentInteraction) -> Result<()> {
        if msg.data.component_type != ComponentType::Button {
            return Ok(());
        }
        if msg.data.custom_id.starts_with("circle") {
            let res = self.circle_manager.handle_button(ctx, &msg).await;
            match res {
                Ok(res) => {
                    let res = msg
                        .create_interaction_response(&ctx.http, |r| {
                            r.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|d| d.content(res).ephemeral(true))
                        })
                        .await;
                    if let Err(why) = res {
                        warn!("Cannot respond to command button: {:?}", why);
                    }
                    return Ok(());
                }
                Err(why) => {
                    warn!("Cannot respond to command button: {:?}", why);
                }
            }
        }
        Ok(())
    }
}
