// use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{SharedState, State, StateError},
    StateEngine,
};

#[derive(Debug)]
pub struct Idle;

impl State<Idle> {
    /// Init a new [`Idle`] state.
    pub fn new(shared: SharedState) -> Self {
        Self {
            private: Idle,
            shared,
        }
    }
    pub async fn run_state(&mut self) -> Option<StateEngine> {
        todo!()
    }
    // async fn run_state(&mut self) -> Result<(), StateError> {
    //     Ok(())
    // }
    pub async fn next(self) -> Option<StateEngine> {
        todo!()
    }
}