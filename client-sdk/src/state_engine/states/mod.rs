//! Module implementing the individual states of the ['StateEngine'].
//!
//! [`StateEngine`]: crate::state_engine::StateEngine.
mod idle;
mod state;

pub use self::{
    idle::Idle,
    state::{SharedState, State, StateCondition, StateError, StateName},
};