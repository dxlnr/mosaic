//! This module provides the states (aka States) of the [`StateMachine`].
//!
//! [`StateMachine`]: crate::state_machine::StateMachine

mod collect;
mod failure;
mod handler;
mod idle;
mod state;
mod shutdown;
mod update;

pub use self::{
    collect::Collect,
    failure::Failure,
    handler::StateHandler,
    idle::{Idle, IdleError},
    state::{State, StateName, StateCondition, SharedState, StateError},
    shutdown::Shutdown,
    update::{Update, UpdateError},
};
