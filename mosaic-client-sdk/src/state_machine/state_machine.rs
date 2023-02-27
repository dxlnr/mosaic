use derive_more::From;

use super::{
    boxed_io,
    Awaiting,
    IntoPhase,
    LocalModelConfig,
    NewRound,
    Phase,
    SendingUpdate,
    SerializableState,
    SharedState,
    State,
    Update,
};
use crate::{settings::PetSettings, ModelStore, MosaicClientTrait, Notify};

/// Outcome of a state machine transition attempt.
#[derive(Debug)]
pub enum TransitionOutcome {
    /// Outcome when the state machine cannot make immediate progress. The state machine
    /// is returned unchanged.
    Pending(StateMachine),
    /// Outcome when a transition occured and the state machine was updated.
    Complete(StateMachine),
}

/// PET state machine.
#[derive(From, Debug)]
pub enum StateMachine {
    /// PET state machine in the "new round" phase
    NewRound(Phase<NewRound>),
    /// PET state machine in the "awaiting" phase
    Awaiting(Phase<Awaiting>),
    // /// PET state machine in the "sum" phase
    // Sum(Phase<Sum>),
    /// PET state machine in the "update" phase
    Update(Phase<Update>),
    // /// PET state machine in the "sum2" phase
    // Sum2(Phase<Sum2>),
    // /// PET state machine in the "sending sum message" phase
    // SendingSum(Phase<SendingSum>),
    /// PET state machine in the "sending update message" phase
    SendingUpdate(Phase<SendingUpdate>),
    // /// PET state machine in the "sending sum2 message" phase
    // SendingSum2(Phase<SendingSum2>),
}

impl StateMachine {
    /// Try to make progress in the PET protocol
    pub async fn transition(self) -> TransitionOutcome {
        match self {
            StateMachine::NewRound(phase) => phase.step().await,
            StateMachine::Awaiting(phase) => phase.step().await,
            // StateMachine::Sum(phase) => phase.step().await,
            StateMachine::Update(phase) => phase.step().await,
            // StateMachine::Sum2(phase) => phase.step().await,
            // StateMachine::SendingSum(phase) => phase.step().await,
            StateMachine::SendingUpdate(phase) => phase.step().await,
            // StateMachine::SendingSum2(phase) => phase.step().await,
        }
    }

    /// Convert the state machine into a serializable data structure so
    /// that it can be saved.
    pub fn save(self) -> SerializableState {
        match self {
            StateMachine::NewRound(phase) => phase.state.into(),
            StateMachine::Awaiting(phase) => phase.state.into(),
            // StateMachine::Sum(phase) => phase.state.into(),
            StateMachine::Update(phase) => phase.state.into(),
            // StateMachine::Sum2(phase) => phase.state.into(),
            // StateMachine::SendingSum(phase) => phase.state.into(),
            StateMachine::SendingUpdate(phase) => phase.state.into(),
            // StateMachine::SendingSum2(phase) => phase.state.into(),
        }
    }

    /// Return the local model configuration of the model that is expected in the update phase.
    pub fn local_model_config(&self) -> LocalModelConfig {
        match self {
            StateMachine::NewRound(ref phase) => phase.local_model_config(),
            StateMachine::Awaiting(ref phase) => phase.local_model_config(),
            // StateMachine::Sum(ref phase) => phase.local_model_config(),
            StateMachine::Update(ref phase) => phase.local_model_config(),
            // StateMachine::Sum2(ref phase) => phase.local_model_config(),
            // StateMachine::SendingSum(ref phase) => phase.local_model_config(),
            StateMachine::SendingUpdate(ref phase) => phase.local_model_config(),
            // StateMachine::SendingSum2(ref phase) => phase.local_model_config(),
        }
    }
}

impl StateMachine {
    /// Instantiate a new PET state machine.
    ///
    /// # Args
    ///
    /// - `settings`: PET settings
    /// - `xaynet_client`: a client for communicating with the Xaynet coordinator
    /// - `model_store`: a store from which the trained model can be
    ///   loaded, when the participant is selected for the update task
    /// - `notifier`: a type that the state machine can use to emit notifications
    pub fn new<X, M, N>(
        settings: PetSettings,
        xaynet_client: X,
        model_store: M,
        notifier: N,
    ) -> Self
    where
        X: MosaicClientTrait + Send + 'static,
        M: ModelStore + Send + 'static,
        N: Notify + Send + 'static,
    {
        let io = boxed_io(xaynet_client, model_store, notifier);
        let state = State::new(Box::new(SharedState::new(settings)), Box::new(Awaiting));
        state.into_phase(io).into()
    }

    /// Restore the PET state machine from the given `state`.
    pub fn restore<X, M, N>(
        state: SerializableState,
        xaynet_client: X,
        model_store: M,
        notifier: N,
    ) -> Self
    where
        X: MosaicClientTrait + Send + 'static,
        M: ModelStore + Send + 'static,
        N: Notify + Send + 'static,
    {
        let io = boxed_io(xaynet_client, model_store, notifier);
        match state {
            SerializableState::NewRound(state) => state.into_phase(io).into(),
            SerializableState::Awaiting(state) => state.into_phase(io).into(),
            // SerializableState::Sum(state) => state.into_phase(io).into(),
            // SerializableState::Sum2(state) => state.into_phase(io).into(),
            SerializableState::Update(state) => state.into_phase(io).into(),
            // SerializableState::SendingSum(state) => state.into_phase(io).into(),
            SerializableState::SendingUpdate(state) => state.into_phase(io).into(),
            // SerializableState::SendingSum2(state) => state.into_phase(io).into(),
        }
    }
}
