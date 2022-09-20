//! Module implementing the individual states of the ['StateEngine'].
//!
//! [`StateEngine`]: crate::state_engine::StateEngine.
mod idle;
mod state;
mod update;

pub use self::{
    idle::Idle,
    state::{SharedState, State, StateError},
    update::Update,
};