use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serenity::prelude::TypeMapKey;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coper {
    #[serde(rename = "_id")]
    pub id: String,
    pub score: i64,
}

impl TypeMapKey for Coper {
    type Value = HashMap<String, Self>;
}
