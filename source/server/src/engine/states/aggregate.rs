use async_trait::async_trait;
use std::io::Error;
use std::{thread, time::Duration};

use crate::{
    engine::{
        states::{Handler, Collect, Shutdown, State, StateCondition, StateName},
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
        if self.shared.round_id() > self.shared.round_params.training_rounds {
            thread::sleep(Duration::from_secs(10));
            Some(StateCondition::<Shutdown>::new(self.shared).into())
        } else {
            Some(StateCondition::<Collect>::new(self.shared).into())
        }
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
