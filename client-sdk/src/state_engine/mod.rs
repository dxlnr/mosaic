//! StateEngine implements the clients protocol logic.
//!
pub mod mpc;
pub mod states;

use derive_more::From;

use crate::{client::{Notifier, grpc::GRPCClient}, state_engine::{mpc::Smpc, states::{Connect, Idle, Update, SharedState, State}}};

/// [`StateEngine`]
#[derive(From)]
pub enum StateEngine {
    /// [`Idle`] state of client.
    Idle(State<Idle>),
    /// [`Connect`]
    Connect(State<Connect>),
    /// [`Update`] state of client.
    Update(State<Update>),
}

impl StateEngine {
    pub fn new(grpc_client: GRPCClient, notifier: Notifier) -> Self {
        let smpc = Smpc::new(grpc_client, notifier);
        let shared = SharedState::new();

        StateEngine::Idle(State::<Idle>::new(shared, smpc, Idle))
    }

    pub async fn next(self) -> Option<StateEngine> {
        match self {
            StateEngine::Idle(state) => state.run_state().await,
            StateEngine::Connect(state) => state.run_state().await,
            StateEngine::Update(state) => state.run_state().await,
        }
    }

    pub async fn run(mut self) -> Option<()> {
        loop {
            self = self.next().await?;
        }
    }
}
