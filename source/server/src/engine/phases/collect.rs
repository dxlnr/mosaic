use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::{
    phases::{Phase, PhaseName, PhaseState, Shutdown},
    Engine, ServerState,
};

/// The collect state.
#[derive(Debug)]
pub struct Collect;

#[async_trait]
impl Phase for PhaseState<Collect> {
    const NAME: PhaseName = PhaseName::Collect;

    async fn perform(&mut self) -> Result<(), Infallible> {
        let mut n = 0;

        loop {
            if n > 5 {
                break;
            }
            n += 1;
            println!("{:?}", n);
            use std::{thread, time};
            thread::sleep(time::Duration::from_secs(1));
        }
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(PhaseState::<Shutdown>::new(self.shared).into())
    }
}

impl PhaseState<Collect> {
    /// Creates a new collect state.
    pub fn new(mut shared: ServerState) -> Self {
        Self {
            private: Collect,
            shared,
        }
    }
}
