use serenity::{
    builder::CreateEmbed,
    model::prelude::{ Embed, GuildChannel},
    utils::Color,
};

#[derive(Default)]
pub enum ResponseType {
    Error,
    Invalid,
    Warning,
    #[default]
    Normal,
    Success,
}

struct ResponseData {
    simple: String,
    embed: String,
    color: Color,
}

pub enum ResponseFormat {
    Simple(ResponseType),
    Embed(ResponseType),
}

pub struct ResponseUtil {
    pub format: ResponseFormat,
}

impl ResponseUtil {
    pub fn new(format: ResponseFormat) -> Self {
        Self { format }
    }

    fn embed(msg: &str, emoji: &str) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed.description(format!("{} | {}", emoji, msg));
        embed
    }

    pub fn build(&self, msg: &str, _type: Option<ResponseType>) -> CreateEmbed {
        let _type = _type.unwrap_or_default();
        let mut res = _type.get_data();
        ResponseUtil::embed(msg, &res.embed)
    }
}

impl ResponseType {
    fn get_data(&self) -> ResponseData {
        match self {
            ResponseType::Error => ResponseData {
                simple: "❌".to_string(),
                embed: "Error".to_string(),
                color: Color::DARK_RED,
            },
            ResponseType::Invalid => ResponseData {
                simple: "🚫".to_string(),
                embed: "Invalid".to_string(),
                color: Color::RED,
            },
            ResponseType::Warning => ResponseData {
                simple: "⚠️".to_string(),
                embed: "⚠️".to_string(),
                color: Color::GOLD,
            },
            ResponseType::Normal => ResponseData {
                simple: "".to_string(),
                embed: "".to_string(),
                color: Color::DARK_GREY,
            },
            ResponseType::Success => ResponseData {
                simple: "✅".to_string(),
                embed: "✅".to_string(),
                color: Color::DARK_GREEN,
            },
        }
    }
}
