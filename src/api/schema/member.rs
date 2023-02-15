use std::collections::HashMap;

use mongodb::bson::DateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serenity::prelude::TypeMapKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: String,
    pub strikes: i64,
    pub last_strike: DateTime,
    pub preferences: Preferences,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub subscribed: bool,
}

impl TypeMapKey for Member {
    type Value = HashMap<String, Member>;
}
