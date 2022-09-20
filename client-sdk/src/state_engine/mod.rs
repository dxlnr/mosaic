//! StateEngine implements the clients protocol logic.
//!
pub mod states;

use derive_more::From;

use crate::{client::EventSender, state_engine::states::{Connect, Idle, Update, SharedState, State}};

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
    pub fn new(event_sender: EventSender) -> Self {
        let shared = SharedState::new(event_sender);

        StateEngine::Idle(State::<Idle>::new(shared, Idle))
    }

    pub async fn next(self) -> Option<Self> {
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
