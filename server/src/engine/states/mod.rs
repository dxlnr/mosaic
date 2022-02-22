//! States module that implements the different individual states the ['Engine'] operates in.
//!
//! [`Engine`]: crate::engine::Engine.
mod aggregate;
mod collect;
pub mod error;
// mod failure;
mod idle;
mod shutdown;
mod state;

pub use self::{
    aggregate::Aggregate,
    collect::Collect,
    idle::Idle,
    shutdown::Shutdown,
    state::{Handler, State, StateCondition, StateName},
};
