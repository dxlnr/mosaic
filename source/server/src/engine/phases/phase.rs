//use std::fmt;

use async_trait::async_trait;
use derive_more::Display;
use std::convert::Infallible;

use crate::engine::Engine;
use crate::engine::ServerState;

/// The name of the current phase.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum PhaseName {
    #[display(fmt = "Init")]
    Init,
    #[display(fmt = "Collect")]
    Collect,
    #[display(fmt = "Shutdown")]
    Shutdown,
}

/// A trait that must be implemented by a state in order to perform its tasks and to move to a next state.
#[async_trait]
pub trait Phase {
    /// The name of the current phase.
    const NAME: PhaseName;

    /// Performs the tasks of this phase.
    async fn perform(&mut self) -> Result<(), Infallible>;

    /// Moves from this phase to the next phase.
    async fn next(self) -> Option<Engine>;
}

pub struct PhaseState<S> {
    pub(in crate::engine) private: S,
    /// Some shared state.
    pub shared: ServerState,
}

impl<S> PhaseState<S>
where
    Self: Phase,
{
    /// Runs the current phase to completion.
    pub async fn run_phase(mut self) -> Option<Engine> {
        let phase = Self::NAME;

        println!("Engine runs phase: {:?}", &phase);

        async move {
            if let Err(_err) = self.perform().await {
                println!("{:?}", "phase error");
            }
            self.next().await
        }
        .await
    }
}
