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
pub type Reaction = &'static str;
pub type Reactions = ArrayVec<[Reaction; 20]>;
pub type StateResult = Result<Action, Box<dyn Error>>;

#[derive(Debug)]
pub enum Action {
    NoChange(State),
    Changed(State),
    BadReact(State),
}

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
pub trait CharacterState: Display + Debug {
    async fn action(self: Box<Self>, database: &Database, reaction: Reaction) -> StateResult;

    async fn reactions(&self, database: &Database) -> Reactions;
}
