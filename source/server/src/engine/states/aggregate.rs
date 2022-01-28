use async_trait::async_trait;
use std::io::Error;

use crate::{
    engine::{
        states::{Handler, Idle, State, StateCondition, StateName},
        utils::features::Features,
        Engine, ServerState,
    },
    message::Message,
    service::error::ServiceError,
};

/// The Aggregate state.
#[derive(Debug)]
pub struct Aggregate {
    features: Features,
}

#[async_trait]
impl State for StateCondition<Aggregate>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Aggregate;

    async fn perform(&mut self) -> Result<(), Error> {
        self.aggregate();

        let global = self.private.features.global.clone();
        self.shared.publisher.broadcast_model(global);
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Idle>::new(self.shared).into())
    }
}

impl StateCondition<Aggregate> {
    /// Creates a new Aggregate state.
    pub fn new(shared: ServerState, features: Features) -> Self {
        Self {
            private: Aggregate { features },
            shared,
        }
    }
    /// Aggreates all the features from collect state into the global model.
    pub fn aggregate(&mut self) {
        self.private.features.add();
        self.private.features.avg();
        
        // self.shared
        //     .features
        //     .increment(&self.shared.round_params.per_round_participants);
        // self.shared.features.flush();
    }
}

#[async_trait]
impl Handler for StateCondition<Aggregate> {
    async fn handle_request(&mut self, _req: Message) -> Result<(), ServiceError> {
        Ok(())
    }
}
