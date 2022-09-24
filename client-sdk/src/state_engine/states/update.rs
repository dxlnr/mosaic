use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{Idle, State, StateCondition, StateError},
    StateEngine,
};

#[derive(Debug)]
pub struct Update;

#[async_trait]
impl StateCondition<Update> for State<Update> {

    async fn perform(&mut self) -> Result<(), StateError> {
        println!("\t\tClient Engine : Update state");
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(State::<Idle>::new(self.shared, self.smpc, Idle).into())
    }
}