use std::collections::HashMap;

use color_eyre::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model::prelude::{
        component::ButtonStyle, interaction::message_component::MessageComponentInteraction,
        Channel, ChannelId, GuildId, ReactionType, RoleId,
    },
    prelude::Context,
};
use tracing::{debug, info, instrument};
use urlencoding::encode;

use crate::{api::schema::circle::Circle, settings::Settings};

pub struct CircleManager {
    join_channel: ChannelId,
    #[allow(dead_code)]
    leader_channel: ChannelId,
    guild_id: GuildId,
}

impl CircleManager {
    #[must_use]
    pub fn new(settings: &Settings) -> Self {
        Self {
            join_channel: ChannelId(settings.circles.join_channel),
            leader_channel: ChannelId(settings.circles.leader_channel),
            guild_id: GuildId(settings.guild),
        }
    }

    #[instrument(skip(self, ctx))]
    pub async fn repost(&self, ctx: &Context) -> Result<()> {
        debug!("Reposting circles");
        let channel = ctx.http.get_channel(self.join_channel.into()).await?;
        self.delete_original(ctx, &channel).await?;
        self.send_header(ctx, &channel).await?;

        let data = ctx.data.read().await;
        let circles = data
            .get::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;

        for c in circles.values() {
            debug!("Posting circle: {}", c.name);
            let (embed, action_row) = self.send_circle_card(ctx, c.clone()).await?;
            channel
                .id()
                .send_message(&ctx.http, |m| {
                    m.components(|c| c.add_action_row(action_row))
                        .set_embed(embed)
                })
                .await?;
        }

        Ok(())
    }

    #[instrument(skip(self, ctx, channel))]
    async fn delete_original(&self, ctx: &Context, channel: &Channel) -> Result<()> {
        let msgs = ctx
            .http
            .get_messages(channel.id().into(), "?limit=50")
            .await?;
        let mut mult_res = Vec::new();
        // Run the delete command on all messages
        for msg in msgs {
            let res = msg.delete(&ctx.http).await;
            mult_res.push(res);
        }
        // Check if any of the deletes failed
        for res in mult_res {
            res?;
        }

        Ok(())
    }

    #[instrument(skip(self, ctx, channel))]
    async fn send_header(&self, ctx: &Context, channel: &Channel) -> Result<()> {
        let circle_header = "https://cdn.discordapp.com/attachments/537776612238950410/826695146250567681/circles.png";

        let yellow_circle =
            "> :yellow_circle: Circles are interest groups made by the community!\n".to_owned();
        let door = "> :door: Join one by reacting to the emoji attached to each.\n".to_owned();
        let crown = "> :crown: You can apply to make your own Circle by filling out this application: <https://apply.acmutd.co/circles>\n".to_owned();
        let text = yellow_circle + &door + &crown;
        let circle_body = text;

        channel
            .id()
            .send_message(&ctx.http, |m| m.content(circle_header))
            .await?;

        channel
            .id()
            .send_message(&ctx.http, |m| m.content(circle_body))
            .await?;
        debug!("Sent header");
        Ok(())
    }

    #[instrument(skip(self, ctx, c))]
    async fn send_circle_card(
        &self,
        ctx: &Context,
        c: Circle,
    ) -> Result<(CreateEmbed, CreateActionRow)> {
        let owner = ctx.http.get_user(c.owner.parse::<u64>()?).await?;
        let member_count = self.get_member_count(ctx, &c).await?;
        let member_count = match member_count {
            0 => "N/A".to_owned(),
            _ => member_count.to_string(),
        };
        let mut circle_reaction = HashMap::new();

        let role = ctx.http.get_guild_roles(self.guild_id.into()).await?;
        let role = role
            .iter()
            .find(|r| r.name == format!("{} {}", c.emoji, c.name))
            .ok_or(eyre::eyre!("Unable to get role"))?;

        let footer_text = format!(
            "Created on {}﹒👑 Owner: {}",
            c.created_on.format("%B %d, %Y"),
            owner.name
        );

        circle_reaction.insert(c.emoji.clone(), c.id.clone());
        let encoded_data = EncodeData {
            name: c.name.clone(),
            circle: c.id.clone(),
            reactions: circle_reaction,
            channel: c.channel.parse::<u64>()?.into(),
        };

        let embed = match is_url(&c.image_url) {
            true => CreateEmbed::default()
                .title(format!("{} {} {} ", c.emoji, c.name, c.emoji))
                .color(role.colour)
                .field("**Role**", format!("<@&{}>", c.id), true)
                .field("**Members**", member_count, true)
                .footer(|f| f.text(footer_text))
                .description(format!("{} {}", encoded_data.encode()?, c.description))
                .thumbnail(c.image_url)
                .clone(),
            false => CreateEmbed::default()
                .title(format!("{} {} {} ", c.emoji, c.name, c.emoji))
                .color(role.colour)
                .field("**Role**", format!("<@&{}>", c.id), true)
                .field("**Members**", member_count, true)
                .footer(|f| f.text(footer_text))
                .description(format!("{} {}", encoded_data.encode()?, c.description))
                .clone(),
        };

        let emoji: ReactionType = c.emoji.clone().try_into()?;

        let join_button = CreateButton::default()
            .label(format!("Join/Leave {}", c.name))
            .custom_id(format!("circle/join/{}", c.id))
            .emoji(emoji)
            .style(ButtonStyle::Primary)
            .clone();

        let about_button = CreateButton::default()
            .label("Learn More")
            .custom_id(format!("circle/about/{}", c.id))
            .style(ButtonStyle::Secondary)
            .disabled(true)
            .clone();
        let action_row = CreateActionRow::default()
            .add_button(join_button)
            .add_button(about_button)
            .clone();

        Ok((embed, action_row))
    }

    #[instrument(skip(self, ctx, c))]
    async fn get_member_count(&self, ctx: &Context, c: &Circle) -> Result<i32> {
        let guild = ctx.http.get_guild(self.guild_id.into()).await?;

        let members = guild.members(&ctx.http, None, None).await?;
        let role = guild
            .role_by_name(format!("{} {}", c.emoji, c.name).as_str())
            .ok_or(eyre::eyre!("Unable to get role"))?;

        let count = members
            .iter()
            .filter(|m| m.roles.contains(&role.id))
            .count() as i32;

        Ok(count)
    }

    #[instrument(skip(self, ctx))]
    pub async fn handle_button(
        &self,
        ctx: &Context,
        int: &MessageComponentInteraction,
    ) -> Result<String> {
        let data = int.data.custom_id.clone();
        let reg = Regex::new(r"circle/([^/]*)/([^/]+)")?;
        let matches = reg
            .captures(&data)
            .ok_or(eyre::eyre!("Unable to get matches"))?;
        let action = matches
            .get(1)
            .ok_or(eyre::eyre!("Unable to get action"))?
            .as_str();
        let circle_id = matches
            .get(2)
            .ok_or(eyre::eyre!("Unable to get circle id"))?
            .as_str();

        info!("Action: {} Circle: {}", action, circle_id);

        let circle = self.get_circle(ctx, circle_id).await?;

        let res = match action {
            "join" => self.handle_join(ctx, &circle, int).await?,
            _ => "Unable to get action".to_owned(),
        };

        Ok(res)
    }

    #[instrument(skip(self, ctx))]
    async fn get_circle(&self, ctx: &Context, circle_id: &str) -> Result<Circle> {
        let data = ctx.data.read().await;
        let circles = data
            .get::<Circle>()
            .ok_or(eyre::eyre!("Unable to get circles"))?;

        let circle = circles
            .get(circle_id)
            .ok_or(eyre::eyre!("Unable to get circle"))?;
        Ok(circle.to_owned())
    }

    #[instrument(skip(self, ctx))]
    async fn handle_join(
        &self,
        ctx: &Context,
        c: &Circle,
        int: &MessageComponentInteraction,
    ) -> Result<String> {
        let mut member = self.guild_id.member(&ctx.http, int.user.id).await?;

        let channel = ctx.http.get_channel(c.channel.parse::<u64>()?).await?;

        let role_id: RoleId = c.id.parse::<u64>()?.into();
        if member.roles.contains(&role_id) {
            member.remove_role(&ctx.http, role_id).await?;
            let res = format!(
                "You have left the {} circle. Thank you for using circles",
                c.name
            );
            Ok(res)
        } else {
            member.add_role(&ctx.http, role_id).await?;
            let res = format!(
                "You have joined the {} circle. Thank you for using circles",
                c.name
            );
            channel
                .id()
                .send_message(&ctx.http, |m| {
                    m.content(format!(
                        "Welcome to the {} circle <@{}>!",
                        c.name, int.user.id
                    ))
                })
                .await?;
            Ok(res)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct EncodeData {
    name: String,
    circle: String,
    reactions: HashMap<String, String>,
    channel: ChannelId,
}

impl EncodeData {
    fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        let encode = &format!("[\u{200B}](http://fake.fake?data={})", encode(&json));
        Ok(encode.to_string())
    }
}

fn is_url(s: &str) -> bool {
    if s.starts_with("http://") || s.starts_with("https://") {
        return true;
    }
    false
}
