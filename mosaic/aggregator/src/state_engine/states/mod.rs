//! States module implementing the individual states the ['StateEngine'] operates on.
//!
//! [`StateEngine`]: crate::state_engine::StateEngine.
mod idle;
mod state;

pub use self::{
    idle::Idle,
    state::{State, StateError, StateName},
};