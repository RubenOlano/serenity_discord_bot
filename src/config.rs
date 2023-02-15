use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub token: String,
    pub prefix: String,
    pub activity: Activity,
    #[serde(rename = "sentryDNS")]
    pub sentry_dns: String,
    #[serde(rename = "databaseURL")]
    pub database_url: String,
    pub firestore: Firestore,
    pub express: Express,
    pub keys: Keys,
    pub response_format: String,
    pub disabled_commands: Vec<Value>,
    pub disabled_categories: Vec<Value>,
    pub guild: String,
    #[serde(rename = "acmLogoURL")]
    pub acm_logo_url: String,
    pub points: Points,
    pub circles: Circles,
    pub channels: Channels,
    pub roles: Roles,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(rename = "type")]
    pub type_field: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Firestore {
    pub project_id: String,
    pub key_filename: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Express {
    pub port: i64,
    pub private_key: String,
    pub cert: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keys {
    pub sheets: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Points {
    pub private_channel: String,
    pub public_channel: String,
    pub staff_role: String,
    pub firebase_root: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Circles {
    pub join_channel: String,
    pub parent_category: String,
    pub leader_channel: String,
    pub remind_cron: String,
    pub remind_threshold_days: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channels {
    pub verification: String,
    pub error: String,
    pub shoutout: String,
    pub roles: String,
    #[serde(rename = "mod")]
    pub mod_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Roles {
    pub member: String,
    pub staff: String,
    pub director: String,
    pub mute: String,
    pub divisions: Divisions,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Divisions {
    pub projects: String,
    pub education: String,
    pub hackutd: String,
}
