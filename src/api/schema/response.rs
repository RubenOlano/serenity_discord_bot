use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};
use serenity::prelude::TypeMapKey;

#[derive(Serialize, Deserialize)]
pub enum ResponsesType {
    Strike,
    Kick,
    Ban,
    Mute,
    Caretaker,
}

impl Display for ResponsesType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponsesType::Strike => write!(f, "strike"),
            ResponsesType::Kick => write!(f, "kick"),
            ResponsesType::Ban => write!(f, "ban"),
            ResponsesType::Mute => write!(f, "mute"),
            ResponsesType::Caretaker => write!(f, "caretaker"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "type")]
    pub type_field: String,
    pub message: String,
}

impl TypeMapKey for Response {
    type Value = HashMap<String, Response>;
}
