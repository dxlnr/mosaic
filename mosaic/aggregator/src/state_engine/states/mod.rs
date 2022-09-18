//! States module implementing the individual states the ['StateEngine'] operates on.
//!
//! [`StateEngine`]: crate::state_engine::StateEngine.
mod collect;
mod failure;
mod handler;
mod idle;
mod shutdown;
mod state;
mod update;

pub use self::{
    collect::Collect,
    failure::Failure,
    handler::StateHandler,
    idle::Idle,
    shutdown::Shutdown,
    state::{SharedState, State, StateCondition, StateError, StateName},
    update::Update,
};
