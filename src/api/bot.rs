use crate::{commands, managers::mongo::Mongo};

// use super::super::managers::firestore::FirestoreManager;
use super::super::settings::Settings;
use color_eyre::Report;
use serenity::{
    async_trait,
    model::prelude::{
        interaction::{Interaction, InteractionResponseType},
        Activity, GuildId, Ready,
    },
    prelude::{Context, EventHandler},
};
use tracing::{info, log::warn};

pub struct Bot {
    pub settings: Settings,
    // pub firestore_manager: FirestoreManager,
    pub mongo_manager: Mongo,
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
        })
        .await
        .unwrap_or_else(|why| {
            warn!("Cannot register commands: {:?}", why);
            Vec::new()
        });

        info!("Commands: registered{:#?}", commands)
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {
            info!("Command: {:?}", cmd.data.name);

            let content = match cmd.data.name.as_str() {
                // "admin" => commands::admin::run(&cmd.data.options, self).await,
                "circle" => commands::circle::run(&cmd.data.options, &ctx, self).await,
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
                        warn!("Cannot respond to command: {:?}", why)
                    }
                }
                Err(why) => {
                    warn!("Cannot respond to command: {:?}", why);
                    if let Err(why) = cmd
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content(why.to_string()).ephemeral(true)
                                })
                        })
                        .await
                    {
                        warn!("Cannot respond to command: {:?}", why)
                    }
                }
            }
        }
    }
}

impl Bot {
    pub async fn new() -> Self {
        let settings = Settings::new();
        // let firestore_manager = FirestoreManager::new().await;
        let mongo_manager = Mongo::new(&settings).await;
        Self {
            settings,
            // firestore_manager,
            mongo_manager,
        }
    }
}
