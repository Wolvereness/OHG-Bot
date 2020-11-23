mod shim;

use serde::{
    Deserialize,
    Serialize,
};
use wither::prelude::*;
use serenity::model::prelude::*;
use wither::bson::oid::ObjectId;
use serenity::prelude::*;
use wither::mongodb::Database;

pub struct DatabaseHandle;

impl TypeMapKey for DatabaseHandle {
    type Value = Database;
}

#[derive(Model, Deserialize, Serialize)]
pub struct DiscordCredentials {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    pub private: String,
    pub public: String,
    pub token: String,
    pub bot_id: String,
    pub prefix: String,
}

#[derive(Model, Deserialize, Serialize, Debug)]
pub struct RoleAssociation {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, with = "shim::Optional", skip_serializing_if="Option::is_none")]
    pub channel: Option<ChannelId>,
    #[serde(default, with = "shim::Optional", skip_serializing_if="Option::is_none")]
    pub server: Option<GuildId>,
    #[serde(with = "shim::Required")]
    pub role: RoleId,
}

#[derive(Model, Deserialize, Serialize, Debug)]
pub struct RoleStatus {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(with = "shim::Required")]
    pub role: RoleId,
}


