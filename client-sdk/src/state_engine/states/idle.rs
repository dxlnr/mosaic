use async_trait::async_trait;
use tokio::signal;
// use tracing::warn;

use crate::state_engine::{
    states::{Connect, State, StateCondition, StateError},
    StateEngine,
};

#[derive(Debug)]
pub struct Idle;

#[async_trait]
impl StateCondition<Idle> for State<Idle> {

    async fn perform(&mut self) -> Result<(), StateError> {
        println!("\t\tClient Engine : Idle state");

        loop {
            tokio::select! {
                biased;

                _ =  signal::ctrl_c() => {
                    break Ok(())
                }
            }
        }
    }

    async fn next(self) -> Option<StateEngine> {
        Some(State::<Connect>::new(self.shared, self.smpc, Connect).into())
    }

    
}