use async_trait::async_trait;
use std::convert::Infallible;
use std::io::Error;
// use tracing::info;

use crate::{
    engine::{
        states::{Handler, Shutdown, State, StateCondition, StateName},
        Engine, ServerState,
    },
    message::Message,
};

/// The Aggregate state.
#[derive(Debug)]
pub struct Aggregate;

#[async_trait]
impl State for StateCondition<Aggregate>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Aggregate;

    async fn perform(&mut self) -> Result<(), Error> {
        println!("{:?}", &self.shared.features);
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Shutdown>::new(self.shared).into())
    }
}

impl StateCondition<Aggregate> {
    /// Creates a new Aggregate state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Aggregate,
            shared,
        }
    }
}

#[async_trait]
impl Handler for StateCondition<Aggregate> {
    async fn handle_request(&mut self, req: Message) -> Result<(), Infallible> {
        Ok(())
    }
}
