use async_trait::async_trait;

use crate::{
    core::{aggregator::features::FeatureMap, model::ModelWrapper},
    db::traits::ModelStorage,
    engine::{
        states::{error::StateError, Collect, State, StateCondition, StateName},
        Cache, Engine, ServerState,
    },
};

#[derive(Debug)]
/// [`Idle`] object representing the idle state.
pub struct Idle;

#[async_trait]
impl State for StateCondition<Idle> {
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), StateError> {
        let global = self
            .shared
            .store
            .get_global_model()
            .await
            .map_err(StateError::IdleError)?;

        let model_wrapper = ModelWrapper::new(
            global.unwrap(),
            self.shared.round_params.dtype,
            self.cache.round_id,
        );
        self.shared.publisher.broadcast_model(model_wrapper);
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(
            StateCondition::<Collect>::new(self.shared, FeatureMap::default(), self.cache).into(),
        )
    }
}

impl StateCondition<Idle> {
    /// Creates a new idle state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Idle,
            shared,
            cache: Cache::default(),
        }
    }
}
