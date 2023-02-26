use std::collections::HashMap;
use std::fmt::Display;

use color_eyre::Result;
use serenity::builder::{CreateActionRow, CreateEmbed};
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::Message;
use serenity::prelude::TypeMapKey;
use uuid::Uuid;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct Report {
    message: Message,
    original_interaction: ApplicationCommandInteraction,
}

impl TypeMapKey for Report {
    type Value = HashMap<Uuid, Report>;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
enum ReportCategories {
    Offensive,
    SpamOrAds,
    IllegalOrNSFW,
    Uncomfortable,
    Other,
}

impl Display for ReportCategories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportCategories::Offensive => write!(f, "Offensive"),
            ReportCategories::SpamOrAds => write!(f, "Spam/Ads"),
            ReportCategories::IllegalOrNSFW => write!(f, "Illegal or NSFW"),
            ReportCategories::Uncomfortable => write!(f, "Uncomfortable"),
            ReportCategories::Other => write!(f, "Other"),
        }
    }
}


#[derive(Default)]
pub struct ReportManager;


impl ReportManager {
    pub async fn handle_init_report(&self, message: &Message, ctx: &Context, cmd: &ApplicationCommandInteraction) -> Result<(CreateEmbed, CreateActionRow)> {
        let report_id = Uuid::new_v4();
        let mut data = ctx.data.write().await;
        let reports = data.get_mut::<Report>().ok_or(eyre::eyre!("Unable to get reports"))?;
        report_id.to_string();
        let mut action_row = CreateActionRow::default();
        for category in ReportCategories::iter() {
            action_row.create_button(|b| {
                b.label(category)
                    .style(ButtonStyle::Primary)
                    .custom_id(format!("report/{}/{}", report_id, category))
            });
        }

        let action_row = action_row.clone();

        let content = format!("[Link to message]({})", message.link());
        let embed = CreateEmbed::default()
            .title(format!("Anonymous report {report_id}"))
            .description("Please select a category from the buttons below").clone();
        reports.insert(report_id, Report { message: message.clone(), original_interaction: cmd.clone() });
        Ok((embed, action_row))
    }
}