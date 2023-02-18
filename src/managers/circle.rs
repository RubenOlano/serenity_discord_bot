use std::collections::HashMap;

use color_eyre::Result;
use serde::{Deserialize, Serialize};
use serenity::{
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    model::prelude::{component::ButtonStyle, Channel, ChannelId, GuildId, ReactionType},
    prelude::Context,
};
use urlencoding::encode;

use crate::{api::schema::circle::Circle, settings::Settings};

pub struct CircleManager {
    join_channel: ChannelId,
    #[allow(dead_code)]
    leader_channel: ChannelId,
    guild_id: GuildId,
}

impl CircleManager {
    pub fn new(settings: &Settings) -> Self {
        Self {
            join_channel: ChannelId(settings.circles.join_channel),
            leader_channel: ChannelId(settings.circles.leader_channel),
            guild_id: GuildId(settings.guild.into()),
        }
    }
    pub async fn repost(&self, ctx: &Context) -> Result<()> {
        let channel = ctx.http.get_channel(self.join_channel.into()).await?;
        self.delete_original(ctx, &channel).await?;
        self.send_header(ctx, &channel).await?;

        let data = ctx.data.read().await;
        let circles = data
            .get::<Circle>()
            .ok_or(eyre::eyre!("Unable to get cache"))?;

        for c in circles.values() {
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

        Ok(())
    }

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

        let footer_text = format!("Created on {}﹒👑 Owner: {}", c.created_on, owner.name);

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
                .to_owned(),
            false => CreateEmbed::default()
                .title(format!("{} {} {} ", c.emoji, c.name, c.emoji))
                .color(role.colour)
                .field("**Role**", format!("<@&{}>", c.id), true)
                .field("**Members**", member_count, true)
                .footer(|f| f.text(footer_text))
                .description(format!("{} {}", encoded_data.encode()?, c.description))
                .to_owned(),
        };

        let emoji: ReactionType = c.emoji.clone().try_into()?;

        let join_button = CreateButton::default()
            .label(format!("Join/Leave {}", c.name))
            .custom_id(format!("circle/join/{}", c.id))
            .emoji(emoji)
            .style(ButtonStyle::Primary)
            .to_owned();

        let about_button = CreateButton::default()
            .label("Learn More")
            .custom_id(format!("circle/about/{}", c.id))
            .style(ButtonStyle::Secondary)
            .disabled(true)
            .to_owned();
        let action_row = CreateActionRow::default()
            .add_button(join_button)
            .add_button(about_button)
            .to_owned();

        Ok((embed, action_row))
    }

    async fn get_member_count(&self, ctx: &Context, c: &Circle) -> Result<i32> {
        let guild = ctx.http.get_guild(self.guild_id.into()).await?;

        let members = guild.members(&ctx.http, None, None).await?;
        let role = guild
            .role_by_name(format!("{} {}", c.emoji, c.name).as_str())
            .ok_or(eyre::eyre!("Unable to get role"))?;

        let mut count = 0;
        for member in members {
            if member.roles.contains(&role.id) {
                count += 1;
            }
        }

        Ok(count)
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
