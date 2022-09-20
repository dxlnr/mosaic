// use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{SharedState, State, StateError},
    StateEngine,
};

#[derive(Debug)]
pub struct Update;

impl State<Update> {
    /// Init a new [`Train`] state.
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Update,
            shared,
        }
    }
    pub async fn run_state(&mut self) -> Option<StateEngine> {
        todo!()
    }

    // async fn process(&mut self) -> Result<(), StateError> {
    //     Ok(())
    // }

    pub async fn next(self) -> Option<StateEngine> {
        todo!()
    }
}