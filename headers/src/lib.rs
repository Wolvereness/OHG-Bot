use std::fmt::{
    Display,
    Debug,
};

use arrayvec::ArrayVec;
use wither::mongodb::Database;
use async_trait::async_trait;
use typetag;

type State = Box<dyn CharacterState>;
type Reaction = &'static str;
type Reactions = ArrayVec<[Reaction; 20]>;

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
    async fn action(self: Box<Self>, database: &Database, reaction: Reaction) -> Action;

    async fn reactions(&self, database: &Database) -> Reactions;
}