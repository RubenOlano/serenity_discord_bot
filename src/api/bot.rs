use crate::{
    api::schema::circle::Circle,
    commands,
    managers::{circle::CircleManager, mongo::Mongo},
};

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
        })
        .await
        .unwrap_or_else(|why| {
            warn!("Cannot register commands: {:?}", why);
            Vec::new()
        });

        info!("Commands: registered{:#?}", commands);

        let mut cursor = self
            .mongo_manager
            .client
            .database("discord")
            .collection::<Circle>("circle")
            .find(None, None)
            .await
            .unwrap();

        let mut data = ctx.data.write().await;
        let c_manager = data.get_mut::<Circle>().unwrap();

        while cursor.advance().await.unwrap() {
            let doc = cursor.current();
            let circle = Circle {
                id: doc.get_str("_id").unwrap().to_string(),
                name: doc.get_str("name").unwrap().to_string(),
                description: doc.get_str("description").unwrap().to_string(),
                image_url: doc.get_str("imageUrl").unwrap().to_string(),
                channel: doc.get_str("channel").unwrap().to_string(),
                emoji: doc.get_str("emoji").unwrap().to_string(),
                owner: doc.get_str("owner").unwrap().to_string(),
                created_on: doc.get_datetime("createdOn").unwrap(),
                sub_channels: doc
                    .get_array("subChannels")
                    .unwrap()
                    .into_iter()
                    .map(|x| x.unwrap().as_str().unwrap().to_string())
                    .collect(),
            };
            c_manager.insert(circle.id.clone(), circle);
        }
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
                        warn!("Cannot respond to command: {:?}", why);
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
                        warn!("Cannot respond to command: {:?}", why);
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
        let circle_manager = CircleManager::new(&settings);
        Self {
            settings,
            // firestore_manager,
            mongo_manager,
            circle_manager,
        }
    }
}
