use async_trait::async_trait;

use crate::{
    core::model::ModelWrapper,
    engine::{
        states::{error::StateError, Collect, State, StateCondition, StateName},
        Engine, ServerState,
    },
    db::traits::ModelStorage,
};

/// The idle state.
#[derive(Debug)]
pub struct Idle;

#[async_trait]
impl State for StateCondition<Idle> {
    const NAME: StateName = StateName::Idle;

    async fn perform(&mut self) -> Result<(), StateError> {
        let global = self
            .shared
            .store
            .get_global_model("global_model")
            .await
            .map_err(StateError::IdleError)?;

        let model_wrapper = ModelWrapper::new(
            global.unwrap(),
            self.shared.round_params.dtype,
            self.shared.round_id,
        );
        self.shared.publisher.broadcast_model(model_wrapper);
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        // if self.shared.round_id() > self.shared.round_params.training_rounds {
        //     Some(StateCondition::<Shutdown>::new(self.shared).into())
        // } else {
        //     Some(StateCondition::<Collect>::new(self.shared).into())
        // }
        Some(StateCondition::<Collect>::new(self.shared).into())
    }
}

impl StateCondition<Idle> {
    /// Creates a new idle state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Idle,
            shared,
        }
    }
}
