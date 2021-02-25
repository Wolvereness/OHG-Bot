#![deny(rust_2018_idioms)]

use std::fmt::Debug;

use arrayvec::ArrayVec;
use wither::{
    Model,
    bson::oid::ObjectId,
    mongodb::Database,
};
use async_trait::async_trait;
use typetag;
use serde::{
    Serialize,
    Deserialize,
};
pub use serenity::builder::CreateEmbed;
pub use once_cell::sync::OnceCell;

mod lazy_db;
pub use lazy_db::LazyDB;

type State = Box<dyn CharacterState>;

#[derive(Debug)]
pub enum Action {
    NoChange(State),
    Changed(State),
    BadReact(State),
}

#[derive(Model, Deserialize, Serialize, Debug)]
pub struct StatePointer {
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    pub contents: Box<dyn CharacterState>,
}

#[derive(Copy, Clone, Debug)]
pub struct StateReaction {
    pub emoji: &'static str,
    pub description: &'static str,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Reactions = ArrayVec<[StateReaction; 20]>;

impl Action {
    pub fn inner(self) -> State {
        match self {
            Action::NoChange(state) => state,
            Action::Changed(state) => state,
            Action::BadReact(state) => state,
        }
    }
}

#[typetag::serde(tag = "state")]
#[async_trait]
pub trait CharacterState: Debug + Send + Sync {
    async fn action(self: Box<Self>, database: &Database, reaction: &str)
        -> Result<Action, Error>;

    async fn reactions(&self, database: &Database)
        -> Result<Reactions, Error>;

    async fn display(&self, database: &Database)
        -> Result<(Reactions, CreateEmbed), Error>;
}

pub fn add_reactions(embed: &mut CreateEmbed, reactions: &Reactions) {
    for StateReaction { emoji, description, } in reactions.iter().copied() {
        embed.field(emoji, description, true);
    }
}
