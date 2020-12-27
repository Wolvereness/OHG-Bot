#![deny(rust_2018_idioms)]

use std::fmt::Debug;

use arrayvec::ArrayVec;
use wither::mongodb::Database;
use async_trait::async_trait;
use typetag;
pub use serenity::builder::CreateEmbed;

type State = Box<dyn CharacterState>;

#[derive(Debug)]
pub enum Action {
    NoChange(State),
    Changed(State),
    BadReact(State),
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
        -> Result<CreateEmbed, Error>;
}

pub fn add_reactions(embed: &mut CreateEmbed, reactions: &Reactions) {
    for StateReaction { emoji, description, } in reactions.iter().copied() {
        embed.field(emoji, description, true);
    }
}
