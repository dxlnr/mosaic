/// Main module as it implements all the key functionality.
/// Aggregation of the global model, keeping track of the training state, publishing protocol events
/// and handling protocol errors.
pub mod channel;
pub mod model;
pub mod states;
pub mod utils;
pub mod watch;

use self::watch::{Publisher, Subscriber};
use derive_more::From;

use crate::{
    engine::{
        channel::{RequestReceiver, RequestSender},
        model::DataType,
        states::{Aggregate, Collect, Idle, Shutdown, StateCondition},
        utils::features::Features,
    },
    settings::{ModelSettings, ProcessSettings},
};

#[derive(From)]
/// ['Engine'] functions as the state machine which handles the whole Federated Learning process
/// on the server side.
pub enum Engine {
    Idle(StateCondition<Idle>),
    Collect(StateCondition<Collect>),
    Aggregate(StateCondition<Aggregate>),
    Shutdown(StateCondition<Shutdown>),
}

impl Engine {
    pub async fn next(self) -> Option<Self> {
        match self {
            Engine::Idle(state) => state.run_state().await,
            Engine::Collect(state) => state.run_state().await,
            Engine::Aggregate(state) => state.run_state().await,
            Engine::Shutdown(state) => state.run_state().await,
        }
    }

    pub async fn run(mut self) -> Option<()> {
        loop {
            self = self.next().await?;
        }
    }
}

/// Handles the ['Engine'] initialization.
pub struct EngineInitializer {
    model_settings: ModelSettings,
    process_settings: ProcessSettings,
}

impl EngineInitializer {
    /// Creates a new [`EngineInitializer`] which sets up the engine running the aggregation algorithm.
    pub fn new(model_settings: ModelSettings, process_settings: ProcessSettings) -> Self {
        EngineInitializer {
            model_settings,
            process_settings,
        }
    }
    /// Initializes the engine and the communication handler.
    pub async fn init(self) -> (Engine, RequestSender, Subscriber) {
        // let global = Model::new(self.model_settings.length);
        let global = Default::default();
        let (publisher, subscriber) = Publisher::new(global);
        let (rx, tx) = RequestSender::new();
        let shared = ServerState::new(
            0,
            RoundParams::new(
                self.process_settings.rounds,
                self.process_settings.participants,
                self.model_settings.data_type,
            ),
            rx,
            publisher,
            Features::new(),
        );
        (
            Engine::Idle(StateCondition::<Idle>::new(shared)),
            tx,
            subscriber,
        )
    }
}

/// Shared ['ServerState']
pub struct ServerState {
    /// Keeps the actual training round updated and in cache.
    pub round_id: u32,
    /// Information about the whole process cached in ['RoundParams'].
    pub round_params: RoundParams,
    /// Field for enabling receiving requests from the client.
    pub rx: RequestReceiver,
    /// Server publishes latest updates.
    pub publisher: Publisher,
    // // /// Holds the actual global model updated after each completed training round.
    // pub global_model: ModelUpdate,
    /// Caches all the incoming messages and their respective data.
    pub features: Features,
}

impl ServerState {
    /// Init new shared server state.
    pub fn new(
        round_id: u32,
        round_params: RoundParams,
        rx: RequestReceiver,
        publisher: Publisher,
        features: Features,
    ) -> Self {
        ServerState {
            round_id,
            round_params,
            rx,
            publisher,
            features,
        }
    }
    /// Sets the round ID to the given value.
    pub fn set_round_id(&mut self, id: u32) {
        self.round_id = id;
    }

    /// Returns the current round ID.
    pub fn round_id(&self) -> u32 {
        self.round_id
    }
}

pub struct RoundParams {
    pub training_rounds: u32,
    pub per_round_participants: u32,
    pub dtype: DataType,
}

impl RoundParams {
    pub fn new(training_rounds: u32, per_round_participants: u32, dtype: DataType) -> Self {
        Self {
            training_rounds,
            per_round_participants,
            dtype,
        }
    }
}
