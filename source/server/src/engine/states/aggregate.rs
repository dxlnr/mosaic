use async_trait::async_trait;
use std::convert::Infallible;
use std::io::Error;
use tracing::info;

use crate::{
    engine::{
        states::{Handler, Shutdown, State, StateCondition, StateName},
        watch::Publisher,
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
        self.aggregate();
        info!(
            "Global Model after round {:?}: {:?}",
            &self.shared.round_id, &self.shared.global_model
        );

        let global = self.shared.global_model.clone();
        self.shared.publisher.broadcast_model(global);
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
    /// Aggreates all the features from collect state into the global model.
    pub fn aggregate(&mut self) {
        self.shared
            .features
            .iter()
            .map(|r| self.shared.global_model.add(&r.data))
            .collect::<Vec<_>>()
            .to_vec();
        self.shared.global_model.avg(&self.shared.participants);
    }
}

#[async_trait]
impl Handler for StateCondition<Aggregate> {
    async fn handle_request(&mut self, _req: Message) -> Result<(), Infallible> {
        Ok(())
    }
}
