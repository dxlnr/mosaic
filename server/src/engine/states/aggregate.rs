use async_trait::async_trait;
use tracing::info;

use crate::{
    core::{
        aggregator::{Aggregator, features::Features},
        model::{DataType, Model, ModelWrapper},
    },
    db::traits::ModelStorage,
    engine::{
        states::{error::StateError, Collect, Handler, Shutdown, State, StateCondition, StateName},
        Engine, ServerState,
    },
    proxy::message::Message,
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

    async fn perform(&mut self) -> Result<(), StateError> {
        self.aggregate();

        let global = self.private.features.global.clone();
        let model_wrapper =
            ModelWrapper::new(global, self.shared.round_params.dtype, self.shared.round_id);
        self.shared.publisher.broadcast_model(model_wrapper);

        info!(
            "updated global model in round {} was published.",
            &self.shared.round_id
        );

        self.shared
            .store
            .set_global_model(&Model::serialize(
                &self.private.features.global,
                &DataType::F32,
            ))
            .await
            .map_err(StateError::IdleError)?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        if self.shared.round_id() > self.shared.round_params.training_rounds {
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
        // self.private.features.add();
        // let test = &self.private.features.aggregator;
        self.private.features.global = self.private.features.aggregator.avg();

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
