#![deny(rust_2018_idioms)]

use std::{
    error::Error,
    fmt::{
        Display,
        Debug,
    },
};

use arrayvec::ArrayVec;
use wither::mongodb::Database;
use async_trait::async_trait;
use typetag;

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

pub type Reactions = ArrayVec<[StateReaction; 20]>;
pub type StateResult = Result<Action, Box<dyn Error>>;

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
pub trait CharacterState: Display + Debug + Send + Sync {
    async fn action(self: Box<Self>, database: &Database, reaction: &str) -> StateResult;

    async fn reactions(&self, database: &Database) -> Reactions;
}
