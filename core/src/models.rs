mod shim;

use serde::{
    Deserialize,
    Serialize,
};
use wither::prelude::*;
use serenity::model::prelude::*;
use wither::bson::oid::ObjectId;
use serenity::prelude::*;

pub use shim::Required as Shim;

#[derive(Model, Deserialize, Serialize)]
pub struct DiscordCredentials {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    pub private: String,
    pub public: String,
    pub token: String,
    pub bot_id: String,
    pub prefix: String,
    #[serde(with = "shim::Required")]
    pub operator: UserId,
}

impl TypeMapKey for DiscordCredentials {
    type Value = DiscordCredentials;
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

#[derive(Model, Deserialize, Serialize, Debug)]
pub struct Runner {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub payload: Option<String>,
    pub command: Vec<String>,
}

pub type Runners = Vec<Runner>;

#[derive(Model, Deserialize, Serialize, Debug)]
pub struct SubSystem {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub start: Runners,
    pub stop: Runners,
}


#[derive(Model, Deserialize, Serialize, Debug)]
pub struct System {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(with = "shim::Required")]
    pub server: GuildId,
    pub boot: Runners,
    pub shutdown: Runners,
    pub sub_system: SubSystem,
}