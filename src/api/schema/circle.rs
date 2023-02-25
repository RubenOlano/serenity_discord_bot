use std::collections::HashMap;

use mongodb::bson::DateTime;
use mongodb::bson::doc;
use serde::Deserialize;
use serde_derive::Serialize;
use serenity::prelude::TypeMapKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Circle {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub emoji: String,
    pub created_on: DateTime,
    pub channel: String,
    pub owner: String,
    pub sub_channels: Vec<String>,
}

impl TypeMapKey for Circle {
    type Value = HashMap<String, Self>;
}
