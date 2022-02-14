use async_trait::async_trait;
use tracing::info;

use crate::{
    core::{
        aggregator::{features::Features, traits::{FedAvg, Aggregator}, Aggregation},
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
// pub struct Aggregate {
//     features: Features,
// }
pub struct Aggregate{
    aggregation: Aggregation,
}

#[async_trait]
impl State for StateCondition<Aggregate>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Aggregate;

    async fn perform(&mut self) -> Result<(), StateError> {
        self.aggregate();

        // let global = self.private.features.global.clone();
        let global = self.shared.global_model.clone();
        let model_wrapper =
            ModelWrapper::new(global, self.shared.round_params.dtype, self.shared.round_id);
        self.shared.publisher.broadcast_model(model_wrapper);

        info!(
            "updated global model in round {} was published.",
            &self.shared.round_id
        );

        // self.shared
        //     .store
        //     .set_global_model(&Model::serialize(
        //         &self.private.features.global,
        //         &DataType::F32,
        //     ))
        //     .await
        //     .map_err(StateError::IdleError)?;
        // Ok(())
        self.shared
            .store
            .set_global_model(&Model::serialize(
                &self.shared.global_model,
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
            private: Aggregate { aggregation: Aggregation::FedAvg(Aggregator::<FedAvg>::new(features)) },
            shared,
        }
    }
    /// Aggreates all the features from collect state into the global model.
    pub fn aggregate(&mut self) {
        // self.private.features.global = self.private.features.aggregator.aggregate(
        //     self.private.features.locals.clone(),
        //     self.private.features.prep_stakes(),
        // );
        self.shared.global_model = self.private.aggregation.aggregate();
    }
}

#[async_trait]
impl Handler for StateCondition<Aggregate> {
    async fn handle_request(&mut self, _req: Message) -> Result<(), ServiceError> {
        Ok(())
    }
}
