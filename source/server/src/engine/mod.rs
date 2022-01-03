/// Main module as it implements all the key functionality.
/// Aggregation of the global model, keeping track of the training state, publishing protocol events
/// and handling protocol errors.
pub mod channel;
pub mod model;
pub mod states;
pub mod watch;

use self::watch::{Publisher, Subscriber};
use derive_more::From;

use crate::{
    engine::{
        channel::{RequestReceiver, RequestSender},
        model::Model,
        states::{Aggregate, Collect, Idle, Shutdown, StateCondition},
    },
    message::Message,
    settings::{ModelSettings, ProcessSettings},
};

#[derive(From)]
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
        let global = Model::new(self.model_settings.length);
        let (publisher, subscriber) = Publisher::new(global);
        let (rx, tx) = RequestSender::new();
        let shared = ServerState::new(
            0,
            0,
            self.process_settings.participants,
            self.process_settings.rounds,
            rx,
            publisher,
            Model::new(self.model_settings.length),
            Features::new(self.model_settings.length),
        );
        (
            Engine::Idle(StateCondition::<Idle>::new(shared)),
            tx,
            subscriber,
        )
    }
}

pub struct ServerState {
    // Keeps training rounds in cache.
    pub round_id: u32,
    pub client_count: u64,
    pub participants: u32,
    pub rounds: u32,

    // Holds the shared model & message states.
    pub rx: RequestReceiver,
    pub publisher: Publisher,
    pub global_model: Model,
    pub features: Features,
}

impl ServerState {
    /// Init new shared server state.
    pub fn new(
        round_id: u32,
        client_count: u64,
        participants: u32,
        rounds: u32,
        rx: RequestReceiver,
        publisher: Publisher,
        global_model: Model,
        features: Features,
    ) -> Self {
        ServerState {
            round_id,
            client_count,
            participants,
            rounds,
            rx,
            publisher,
            global_model,
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

pub struct ClientState {
    /// counts the number of client updates received.
    pub count: u64,
    pub model_length: usize,
}

pub struct Features {
    /// keeps msgs in cache that have been received by the clients.
    pub msgs: Vec<Message>,
    /// keeps track of the number of msgs received by the clients.
    pub factor: u32,
    /// Will store the overall averaged vector of all messages.
    pub global: Vec<f64>,
}

impl Features {
    /// Instantiates new ['Features'] object.
    pub fn new(length: usize) -> Self {
        Features {
            msgs: Vec::new(),
            factor: 0,
            global: vec![0.0; length],
        }
    }
    /// Increment the factor which holds the number of received messages from previous.
    fn increment(&mut self, count: &u32) {
        self.factor += count;
    }

    /// Elementwise addition of (all) single msgs to the global field.
    pub fn add(&mut self) {
        if self.factor != 0 {
            self.global = self
                .global
                .iter()
                .map(|x| x * self.factor as f64)
                .collect::<Vec<_>>()
                .to_vec();
        }
        self.msgs
            .iter()
            .map(|r| {
                self.global = self
                    .global
                    .iter()
                    .zip(&r.data)
                    .map(|(s, x)| s + x)
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
    }
    pub fn avg(&mut self, participants: &u32, round_id: &u32) {
        self.global = self
            .global
            .iter()
            .map(|x| x / (*participants * *round_id) as f64)
            .collect::<Vec<_>>()
            .to_vec();
    }

    /// Removing all messages from previous training round.
    pub fn flush(&mut self) {
        self.msgs.clear();
    }
}
