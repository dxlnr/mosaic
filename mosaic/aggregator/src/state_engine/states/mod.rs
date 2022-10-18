//! This module provides the states (aka States) of the [`StateMachine`].
//!
//! [`StateMachine`]: crate::state_machine::StateMachine

mod collect;
mod failure;
mod handler;
mod idle;
mod shutdown;
mod state;
mod unmask;
mod update;

pub use self::{
    collect::Collect,
    failure::Failure,
    handler::{MessageCounter, StateHandler},
    idle::{Idle, IdleError},
    shutdown::Shutdown,
    state::{SharedState, State, StateCondition, StateError, StateName},
    unmask::{Unmask, UnmaskError},
    update::{Update, UpdateError},
};
