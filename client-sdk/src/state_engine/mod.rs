//! StateEngine implements the clients protocol logic.
//!
pub mod smpc;
pub mod states;

use derive_more::From;

use crate::{
    client::{grpc::GRPCClient, EventSender},
    state_engine::{
        smpc::Smpc,
        states::{IntoState, Idle, NewTask, SharedState, State, StateCondition, Update},
    },
};

/// [`TransitionState`] of the [`StateEngine`]. 
/// 
#[derive(Debug)]
pub enum TransitionState {
    /// Outcome when the state machine cannot make immediate progress. The state machine
    /// is returned unchanged.
    Pending(StateEngine),
    /// Outcome when a transition occured and the state machine was updated.
    Complete(StateEngine),
}

/// [`StateEngine`]
#[derive(From, Debug)]
pub enum StateEngine {
    /// [`Idle`] state of client.
    Idle(State<Idle>),
    /// [`Connect`]
    NewTask(State<NewTask>),
    /// [`Update`] state of client.
    Update(State<Update>),
}

impl StateEngine {
    pub fn new(grpc_client: GRPCClient, event_sender: EventSender) -> Self {
        let smpc = Smpc::new(grpc_client, event_sender);
        // let shared = SharedState::new();

        // StateEngine::Idle(State::<Idle>::new(shared, smpc, Idle))
        let state = State::new(SharedState::new(), smpc, Idle);
        state.into_state().into()
    }

    pub async fn next(self) -> TransitionState {
        match self {
            StateEngine::Idle(state) => state.proceed().await,
            StateEngine::NewTask(state) => state.proceed().await,
            StateEngine::Update(state) => state.proceed().await,
        }
    }

    // pub async fn run(mut self) -> Option<()> {
    //     loop {
    //         self = self.next().await?;
    //     }
    // }
}
