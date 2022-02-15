use async_trait::async_trait;
use tracing::info;

use crate::{
    core::{
        aggregator::{
            features::Features,
            traits::{Aggregator, FedAvg},
            Aggregation,
        },
        model::{DataType, Model, ModelWrapper},
    },
    db::traits::ModelStorage,
    engine::{
        states::{error::StateError, Collect, Handler, Shutdown, State, StateCondition, StateName},
        Cache, Engine, ServerState,
    },
    proxy::message::Message,
    service::error::ServiceError,
};

/// The Aggregate state.
#[derive(Debug)]
pub struct Aggregate {
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

        let global = self.cache.global_model.clone();
        let model_wrapper =
            ModelWrapper::new(global, self.shared.round_params.dtype, self.cache.round_id);
        self.shared.publisher.broadcast_model(model_wrapper);

        info!(
            "updated global model in round {} was published.",
            &self.cache.round_id
        );
        self.shared
            .store
            .set_global_model(&Model::serialize(&self.cache.global_model, &DataType::F32))
            .await
            .map_err(StateError::IdleError)?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        if self.cache.round_id() > self.shared.round_params.training_rounds {
            Some(StateCondition::<Shutdown>::new(self.shared, self.cache).into())
        } else {
            Some(StateCondition::<Collect>::new(self.shared, self.cache).into())
        }
    }
}

impl StateCondition<Aggregate> {
    /// Creates a new Aggregate state.
    pub fn new(shared: ServerState, cache: Cache, features: Features) -> Self {
        Self {
            private: Aggregate {
                aggregation: Aggregation::FedAvg(Aggregator::<FedAvg>::new(features)),
            },
            shared,
            cache,
        }
    }
    /// Aggreates all the features from collect state into the global model.
    pub fn aggregate(&mut self) {
        let (global, m_t, v_t) = self.private.aggregation.aggregate();
        self.cache.global_model = global;
        self.cache.m_t = m_t;
        self.cache.v_t = v_t;
    }
}

#[async_trait]
impl Handler for StateCondition<Aggregate> {
    async fn handle_request(&mut self, _req: Message) -> Result<(), ServiceError> {
        Ok(())
    }
}
