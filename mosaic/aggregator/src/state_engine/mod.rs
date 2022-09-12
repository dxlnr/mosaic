//! The state engine that controls the execution of the aggregation protocol.
//!
//! # 
//! The implementation resembles a finite state machine which allows to keep state with in 
//! a single `Aggregator` and perform the steps of the protocol in that way.
//! 
//! # Engine states
//!
pub mod channel;
pub mod event;
pub mod init;
pub mod states;

use derive_more::From;

use crate::{
    state_engine::{
        states::{Collect, Idle, StateCondition, Update},
    },
};

#[derive(From)]
/// [`StateEngine`] functions as the state machine which handles the progress of the `Aggregator`
/// and keep its state.
///
pub enum StateEngine {
    /// [`Idle`] state.
    Idle(StateCondition<Idle>),
    /// [`Collect`] state.
    Collect(StateCondition<Collect>),
    /// [`Update`] state.
    Update(StateCondition<Update>),
}

impl StateEngine {
    pub async fn next(self) -> Option<Self> {
        todo!()
    }

    pub async fn run(mut self) -> Option<()> {
        loop {
            self = self.next().await?;
        }
    }
}