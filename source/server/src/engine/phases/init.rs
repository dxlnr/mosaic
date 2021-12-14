use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::{
    phases::{Collect, Phase, PhaseName, PhaseState},
    Engine, ServerState,
};

/// The init state.
#[derive(Debug)]
pub struct Init;

#[async_trait]
impl Phase for PhaseState<Init> {
    const NAME: PhaseName = PhaseName::Init;

    async fn perform(&mut self) -> Result<(), Infallible> {
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(PhaseState::<Collect>::new(self.shared).into())
    }
}

impl PhaseState<Init> {
    /// Creates a new idle state.
    pub fn new(mut shared: ServerState) -> Self {
        // Since some events are emitted very early, the round id must
        // be correct when the idle phase starts. Therefore, we update
        // it here, when instantiating the idle PhaseState.
        shared.set_round_id(shared.round_id() + 1);
        Self {
            private: Init,
            shared,
        }
    }
}
