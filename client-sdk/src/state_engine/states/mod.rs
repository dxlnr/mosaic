//! Module implementing the individual states of the ['StateEngine'].
//!
//! [`StateEngine`]: crate::state_engine::StateEngine.
mod new_task;
mod idle;
mod state;
mod update;

pub use self::{
    new_task::NewTask,
    idle::Idle,
    state::{IntoNextState, SharedState, State, StateCondition, StateError},
    update::Update,
};
