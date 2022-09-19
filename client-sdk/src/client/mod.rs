pub mod grpc;

use thiserror::Error;
use tokio::{runtime::Runtime, sync::mpsc};
use tracing::debug;

use crate::{client::grpc::GRPCClient, configs::Conf, state_engine::StateEngine};

use self::grpc::GRPCClientError;

pub enum Event {
    /// Event emitted when the client is done with its task.
    Idle,
}

/// An [`EventReceiver`] for events emitted by the clients internal [`StateEngine`].
pub struct EventReceiver(mpsc::Receiver<Event>);

impl EventReceiver {
    /// Create a new event sender and receiver.
    fn new() -> (Self, EventSender) {
        let (tx, rx) = mpsc::channel(10);
        (Self(rx), EventSender(tx))
    }
}

/// [`EventSender`] that is passed to the client internal [`StateEngine`].
pub struct EventSender(mpsc::Sender<Event>);

impl EventSender {
    fn send(&mut self, event: Event) {
        if let Err(err) = self.0.try_send(event) {
            debug!("Emitting an event to the client failed: {}", err);
        }
    }
}

/// [`Store`]: API for external Storage.
///
#[derive(Default, Clone)]
struct Store {}

impl Store {
    /// Init new [`Store`] API for the client.
    pub fn new() -> Self {
        Self {}
    }
}

/// Clients task data structure.
///
#[derive(Clone, Debug, Copy)]
pub enum Task {
    /// The client performs model training.
    Train,
    /// No task is currently on the line.
    None,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("The initialization of the clients runtime {:?} failed.", _0)]
    Runtime(std::io::Error),
    #[error("gRPC client initialization failed: {:?}", _0)]
    Grpc(GRPCClientError),
}

/// [`Client`]
///
/// The client holds an internal [`StateEngine`] that executes the FL protocol.
///
pub struct Client {
    /// Internal [`StateEngine`] of the client.
    engine: StateEngine,
    /// Receiver for the events emitted by the [`StateEngine`].
    event_recv: EventReceiver,
    /// Storage API for the external device storage where configs, model &
    /// trainings data is fetched from.
    store: Store,
    /// Async runtime to execute the [`StateEngine`].
    ///
    /// The runtime is connected to an internal device Jobscheduler,
    /// which is responsible to call tasks when the device is idle and ready.
    runtime: Runtime,
    /// The participant current task
    task: Task,
    /// [`GRPCClient`] handles the communication to the server.
    ///
    /// Implements the underlying msflp protocol for the client side.
    grpc_client: GRPCClient,
}

impl Client {
    pub async fn init(conf: Conf) -> Result<Self, ClientError> {
        let (event_recv, event_sender) = EventReceiver::new();
        let store = Store::new();
        let engine = StateEngine::new();
        let grpc_client = GRPCClient::new(conf.api.server_address)
            .await
            .map_err(|err| ClientError::Grpc(err))?;
        Self::try_init(engine, event_recv, store, grpc_client)
    }

    pub fn restore() {}

    fn try_init(
        engine: StateEngine,
        event_recv: EventReceiver,
        store: Store,
        grpc_client: GRPCClient,
    ) -> Result<Self, ClientError> {
        let mut client = Self {
            runtime: Self::runtime()?,
            engine,
            event_recv,
            store,
            task: Task::None,
            grpc_client,
        };
        Ok(client)
    }

    fn runtime() -> Result<Runtime, ClientError> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(ClientError::Runtime)
    }
}
