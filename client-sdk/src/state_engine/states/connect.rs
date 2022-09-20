use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{State, StateCondition, StateError, Update},
    StateEngine,
};

#[derive(Debug)]
pub struct Connect;

#[async_trait]
impl StateCondition<Connect> for State<Connect> {

    async fn perform(&mut self) -> Result<(), StateError> {
        println!("\t\tClient Engine : Connect state");
        Ok(())
    }

    async fn next(self) -> Option<StateEngine> {
        Some(State::<Update>::new(self.shared, Update).into())
    }
}