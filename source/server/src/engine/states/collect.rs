use async_trait::async_trait;
use std::io::Error;

use crate::{
    engine::{
        states::{Aggregate, Handler, State, StateCondition, StateName},
        Engine, ServerState,
    },
    message::Message,
};

/// The collect state.
#[derive(Debug)]
pub struct Collect;

#[async_trait]
impl State for StateCondition<Collect>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), Error> {
        self.process().await?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Aggregate>::new(self.shared).into())
    }
}

impl StateCondition<Collect> {
    /// Creates a new collect state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Collect,
            shared,
        }
    }
    /// Add message to feature list.
    fn add(&mut self, req: Message) -> Result<(), Error> {
        Ok(self.shared.features.msgs.push(req))
    }
}

#[async_trait]
impl Handler for StateCondition<Collect> {
    async fn handle_request(&mut self, req: Message) -> Result<(), Error> {
        self.add(req)
    }
}
