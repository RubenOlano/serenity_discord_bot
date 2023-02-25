use std::collections::HashMap;

use firestore::FirestoreTimestamp;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serenity::prelude::TypeMapKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: String,
    pub strikes: i64,
    pub last_strike: FirestoreTimestamp,
    pub preferences: Preferences,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub subscribed: bool,
}

impl TypeMapKey for Member {
    type Value = HashMap<String, Self>;
}
