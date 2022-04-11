//! Main module that handles the core process sequences. It is essentially implemented as a state machine.
//! 
//! The state machine performes aggregation of the global model, keeping track of the training state, publishing protocol events
//! and handling protocol errors.
pub mod channel;
pub mod states;
pub mod watch;

use self::watch::{Publisher, Subscriber};
use derive_more::From;
use displaydoc::Display;
use thiserror::Error;
use tracing::log::warn;

use crate::{
    core::{aggregator::traits::Scheme, model::{DataType, Model, ModelUpdate}},
    db::s3::{S3Client, StorageError},
    engine::{
        channel::{RequestReceiver, RequestSender},
        states::{Aggregate, Collect, Idle, Shutdown, StateCondition},
    },
    rest::{
        client::HttpClient,
        stats::{Stats, StatsUpdate},
    },
    settings::{JobSettings, ModelSettings, ProcessSettings, S3Settings},
};

#[derive(From)]
/// [`Engine`] functions as the state machine which handles the whole Federated Learning process
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

/// Errors occuring during the initialization process and the [`Engine`].
#[derive(Debug, Display, Error)]
pub enum InitError {
    /// Initialization of storage connection failed: {0}
    StorageInit(StorageError),
}

/// Handles the ['Engine'] initialization.
pub struct EngineInitializer {
    job_settings: JobSettings,
    model_settings: ModelSettings,
    process_settings: ProcessSettings,
    s3_settings: S3Settings,
}

impl EngineInitializer {
    /// Creates a new [`EngineInitializer`] which sets up the engine running the aggregation algorithm.
    pub fn new(
        job_settings: JobSettings,
        model_settings: ModelSettings,
        process_settings: ProcessSettings,
        s3_settings: S3Settings,
    ) -> Self {
        EngineInitializer {
            job_settings,
            model_settings,
            process_settings,
            s3_settings,
        }
    }
    /// Initializes the engine and the communication handler.
    pub async fn init(self) -> Result<(Engine, RequestSender, Subscriber), InitError> {
        let (publisher, subscriber) = Publisher::new(ModelUpdate::None, StatsUpdate::None);
        let (rx, tx) = RequestSender::new();
        let store = self
            .init_storage(self.s3_settings.clone())
            .await
            .map_err(InitError::StorageInit)?;

        let shared = ServerState::new(
            RoundParams::new(
                self.process_settings.rounds,
                self.process_settings.participants,
                self.model_settings.data_type,
                self.process_settings.strategy,
            ),
            rx,
            publisher,
            store,
            HttpClient::new(self.job_settings),
        );
        Ok((
            Engine::Idle(StateCondition::<Idle>::new(shared)),
            tx,
            subscriber,
        ))
    }
    pub async fn init_storage(&self, s3_settings: S3Settings) -> Result<S3Client, StorageError> {
        let s3 = S3Client::new(s3_settings).await?;
        match s3.check_conn().await {
            Ok(()) => s3.clone().create_bucket().await?,
            Err(e) => warn!("{}", e),
        }
        Ok(s3)
    }
}

/// Shared static ['ServerState']
pub struct ServerState {
    /// Information about the whole process cached in ['RoundParams'].
    pub round_params: RoundParams,
    /// Field for enabling receiving requests from the client.
    pub rx: RequestReceiver,
    /// Server publishes latest updates.
    pub publisher: Publisher,
    /// Shared storage state. For now it is a s3 Client which holds the storage bucket.
    pub store: S3Client,
    /// HTTP client.
    pub http_client: HttpClient,
}

impl ServerState {
    /// Init new shared server state.
    pub fn new(
        round_params: RoundParams,
        rx: RequestReceiver,
        publisher: Publisher,
        store: S3Client,
        http_client: HttpClient,
    ) -> Self {
        ServerState {
            round_params,
            rx,
            publisher,
            store,
            http_client,
        }
    }
}

#[derive(Debug, Default)]
/// ['Cache'] that holds the state from previous round in order to allow
/// sensible aggregation.
pub struct Cache {
    /// Keeps the actual training round updated and in cache.
    pub round_id: u32,
    /// Holds the actual global model updated after each completed training round.
    pub global_model: Model,
    /// Holds the m_t variable from the previous aggregation round.
    pub m_t: Model,
    /// Holds the v_t variable from the previous aggregation round.
    pub v_t: Model,
    /// Holds all the metrics while the engine performing the federated learning process.
    pub stats: Stats,
}
impl Cache {
    /// Init new shared server state.
    pub fn new(round_id: u32, global_model: Model, m_t: Model, v_t: Model, stats: Stats) -> Self {
        Self {
            round_id,
            global_model,
            m_t,
            v_t,
            stats,
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
    /// Returns parts of the stats correlated to a certain round_id
    pub fn get_stats_with_round_id(&mut self) -> Stats {
        let filtered = self
            .stats
            .msgs
            .iter()
            .filter(|msg| msg.round_id == self.round_id)
            .cloned()
            .collect::<Vec<_>>();
        Stats { msgs: filtered }
    }
}

pub struct RoundParams {
    /// States how many iterations should be made.
    pub training_rounds: u32,
    /// Sets the amount of participants in each iteratioin.
    pub per_round_participants: u32,
    /// Specifies the Data type of the model. Crucial for serde operations.
    pub dtype: DataType,
    /// Sets the aggregation strategy.
    pub strategy: Scheme,
}

impl RoundParams {
    pub fn new(
        training_rounds: u32,
        per_round_participants: u32,
        dtype: DataType,
        strategy: Scheme,
    ) -> Self {
        Self {
            training_rounds,
            per_round_participants,
            dtype,
            strategy,
        }
    }
}
