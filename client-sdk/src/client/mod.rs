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
    Shutdown,
}

/// An [`EventReceiver`] for events emitted by the clients internal [`StateEngine`].
pub struct EventReceiver(mpsc::Receiver<Event>);

impl EventReceiver {
    /// Create a new event sender and receiver.
    fn new() -> (Self, Notifier) {
        let (tx, rx) = mpsc::channel(10);
        (Self(rx), Notifier(tx))
    }

    /// Pop the next event. If no event has been received, return `ClientError::EventsError`.
    fn next(&mut self) -> Result<Event, ClientError> {
        let next = self
            .0
            .try_recv()
            .map_err(|err| ClientError::EventsError(err))?;
        Ok(next)
    }
}

/// [`Notifier`] that is passed to the client internal [`StateEngine`].
pub struct Notifier(mpsc::Sender<Event>);

impl Notifier {
    fn send(&mut self, event: Event) {
        if let Err(err) = self.0.try_send(event) {
            debug!("Emitting an event to the client failed: {}", err);
        }
    }
    pub fn connect(&mut self) {
        self.send(Event::Connect)
    }
    pub fn new_task(&mut self) {
        self.send(Event::NewTask)
    }
    pub fn update(&mut self) {
        self.send(Event::Update)
    }
    pub fn get_global_model(&mut self) {
        self.send(Event::GetGlobalModel)
    }
    pub fn idle(&mut self) {
        self.send(Event::Idle)
    }
    pub fn shutdown(&mut self) {
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
    /// Connect
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
    EventsError(mpsc::error::TryRecvError),
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
    // /// address
    // server_address: &'static str,
    conf: Conf,
}

impl Client {
    pub fn init(conf: Conf) -> Result<Self, ClientError> {
        let server_endpoint = std::borrow::Cow::from(conf.api.server_address.to_string());
        println!("\tClient::init : start.");


        let (event_recv, event_sender) = EventReceiver::new();
        println!("\tClient::init : EventReceiver ready.");
        let store = Store::new();
        println!("\tClient::init : Store ready.");
        let grpc_client = GRPCClient::new(conf.api.server_address.to_string());
        println!("\tClient::init : GRPC wrapper ready.");
        // let grpc_client = GRPCClient::new(conf.api.server_address)
        //     .await
        //     .map_err(|err| ClientError::Grpc(err))?;
        let engine = StateEngine::new(grpc_client.clone(), event_sender);
        println!("\tClient::init : Engine ready.");

        // Self::try_init(engine, event_recv, store, grpc_client, conf)
        Self::try_init(engine, event_recv, store, grpc_client, conf)
    }

    pub fn restore() {}

    fn try_init(
        engine: StateEngine,
        event_recv: EventReceiver,
        store: Store,
        grpc_client: GRPCClient,
        conf: Conf,
    ) -> Result<Self, ClientError> {
        let mut client = Self {
            runtime: Self::runtime()?,
            engine: Some(engine),
            event_recv,
            store,
            task: Task::None,
            grpc_client,
            conf,
        };
        println!("\tClient::try_init : Client object instantiated.");
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
            println!("\t  Client::process : In process loop.");
            // println!("\t  Client::process : Match: ");
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
                Err(err) => {
                    println!("\t  Client::process : error matched: {:?}.", &err);
                    debug!("{:?}", err);
                    break
                }
            }
        }
    }

    pub fn step(&mut self) {
        println!("\t  Client::step : .");
        let state_engine = self.engine.take().expect("unexpected engine failure.");
        let progress = self.runtime.block_on(async { state_engine.next().await });

        self.process();
    }
}
