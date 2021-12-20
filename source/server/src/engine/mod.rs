/// Main module as it implements all the key functionality.
/// Aggregation of the global model, keeping track of the training state, publishing protocol events
/// and handling protocol errors.
pub mod channel;
pub mod model;
pub mod states;

// use crate::db::Db;
use derive_more::From;
use std::sync::{Arc, Mutex};

use crate::{
    engine::{
        channel::{RequestReceiver, RequestSender},
        states::{Collect, Idle, Shutdown, StateCondition},
    },
    settings::{ModelSettings, ProcessSettings},
};

// use std::convert::Infallible;
#[derive(From)]
pub enum Engine {
    Idle(StateCondition<Idle>),
    Collect(StateCondition<Collect>),
    Shutdown(StateCondition<Shutdown>),
    // Aggregate,
}

impl Engine {
    pub async fn next(self) -> Option<Self> {
        match self {
            Engine::Idle(state) => state.run_state().await,
            Engine::Collect(state) => state.run_state().await,
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

    pub async fn init(self) -> (Engine, RequestSender) {
        let (rx, tx) = RequestSender::new();
        let shared = ServerState::new(
            0,
            0,
            self.process_settings.participants,
            rx,
            Arc::new(Mutex::new(Model::new(self.model_settings.length))),
            Vec::new(),
        );
        (Engine::Idle(StateCondition::<Idle>::new(shared)), tx)
    }
}

pub struct ServerState {
    // Keeps training rounds in cache.
    pub round_id: u64,
    pub client_count: u64,
    pub participants: u32,
    //pub client_params: ClientState,

    // Holds the shared model & message states.
    pub rx: RequestReceiver,
    pub global_model: Arc<Mutex<Model>>,
    pub features: Vec<Model>,
}

impl ServerState {
    /// Init new shared server state.
    pub fn new(
        round_id: u64,
        client_count: u64,
        participants: u32,
        rx: RequestReceiver,
        global_model: Arc<Mutex<Model>>,
        features: Vec<Model>,
    ) -> Self {
        ServerState {
            round_id,
            client_count,
            participants,
            rx,
            global_model,
            features,
        }
    }
    /// Sets the round ID to the given value.
    pub fn set_round_id(&mut self, id: u64) {
        self.round_id = id;
    }

    /// Returns the current round ID.
    pub fn round_id(&self) -> u64 {
        self.round_id
    }
}

pub struct ClientState {
    // counts the number of client updates received.
    pub count: u64,
    pub model_length: usize,
}

// use num::{
//     bigint::BigInt,
//     rational::Ratio,
//     // traits::{float::FloatCore, identities::Zero, ToPrimitive},
// };
// use serde::{Deserialize, Serialize};
#[derive(Default, Debug, Clone, PartialEq)]
/// A representation of a machine learning model as vector object.
// pub struct Model(Vec<Ratio<BigInt>>);
pub struct Model(Vec<Vec<u8>>);

impl std::convert::AsRef<Model> for Model {
    fn as_ref(&self) -> &Model {
        self
    }
}

impl Model {
    /// Instantiates a new empty model.
    pub fn new(length: usize) -> Self {
        Model(vec![vec![0; 8]; length])
    }
    /// Returns the number of weights/parameters of a model.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    // /// Creates an iterator that yields references to the weights/parameters of a model.
    // pub fn iter(&self) -> Iter<f64> {
    //     self.0.iter()
    // }
}
