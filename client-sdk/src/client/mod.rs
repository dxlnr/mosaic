pub mod grpc;

use thiserror::Error;
use tokio::{runtime::Runtime, sync::mpsc};
use tracing::debug;

use crate::{client::grpc::GRPCClient, configs::Conf, state_engine::StateEngine};

use self::grpc::GRPCClientError;

pub enum Event {
    /// Client is on hold and waiting for instruction.
    Idle,
    /// Connect
    Connect,
    /// Client device has been selected for plan-determined model updates and metrics.
    /// 
    NewTask,
    /// Get the latest global model when selected for participation in the next
    /// training round.
    GetGlobalModel,
    /// Update
    Update,
    /// Stops the client and shuts it down.
    Shutdown
}

/// An [`EventReceiver`] for events emitted by the clients internal [`StateEngine`].
pub struct EventReceiver(mpsc::Receiver<Event>);

impl EventReceiver {
    /// Create a new event sender and receiver.
    fn new() -> (Self, EventSender) {
        let (tx, rx) = mpsc::channel(10);
        (Self(rx), EventSender(tx))
    }

    /// Pop the next event. If no event has been received, return `ClientError::EventsError`.
    fn next(&mut self) -> Result<Event, ClientError> {
        let next = self.0.try_recv().map_err(|err| ClientError::EventsError(err))?;
        Ok(next)
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
    fn new_task(&mut self) {
        self.send(Event::NewTask)
    }
    fn update(&mut self) {
        self.send(Event::Update)
    }
    fn get_global_model(&mut self) {
        self.send(Event::GetGlobalModel)
    }
    fn idle(&mut self) {
        self.send(Event::Idle)
    }
    fn shutdown(&mut self) {
        self.send(Event::Shutdown)
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
    ///
    Connect,
    /// The client performs model training.
    Update,
    /// No task is currently on the line.
    None,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("The initialization of the clients runtime {:?} failed.", _0)]
    Runtime(std::io::Error),
    #[error("gRPC client initialization failed: {:?}", _0)]
    Grpc(GRPCClientError),
    #[error("Communication channel is dropped for client.")]
    EventsError(mpsc::error::TryRecvError)
}

/// [`Client`]
///
/// The client holds an internal [`StateEngine`] that executes the FL protocol.
///
pub struct Client {
    /// Internal [`StateEngine`] of the client.
    engine: Option<StateEngine>,
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
    pub fn init(conf: Conf) -> Result<Self, ClientError> {
        let server_endpoint = conf.api.server_address.to_string();

        let (event_recv, event_sender) = EventReceiver::new();
        let store = Store::new();
        let grpc_client = GRPCClient::new(conf.api.server_address.to_string());
        // let grpc_client = GRPCClient::new(conf.api.server_address)
        //     .await
        //     .map_err(|err| ClientError::Grpc(err))?;
        let engine = StateEngine::new(event_sender);

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
            engine: Some(engine),
            event_recv,
            store,
            task: Task::None,
            grpc_client,
        };
        client.process();
        Ok(client)
    }

    fn runtime() -> Result<Runtime, ClientError> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(ClientError::Runtime)
    }

    /// Return the participant current task
    pub fn task(&self) -> Task {
        self.task
    }

    // pub fn set_model(&mut self) {
    //     todo!()
    // }

    /// Loop incoming [`Event`] calls.
    fn process(&mut self) {
        loop {
            match self.event_recv.next() {
                Ok(Event::Idle) => {
                    self.task = Task::None;
                }
                Ok(Event::Connect) => {
                    self.task = Task::Connect;
                }
                Ok(Event::Update) => {
                    self.task = Task::Update;
                }
                Ok(Event::NewTask) => {}
                Ok(Event::GetGlobalModel) => {}
                Ok(Event::Shutdown) => {}
                Err(err) => debug!("{:?}", err),
            }
        }
    }

    pub fn go(&mut self) {
        let state_engine = self.engine.take().expect("unexpected engine failure.");
        let progress = self
            .runtime
            .block_on(async { state_engine.next().await });

        self.process();
    }
}
