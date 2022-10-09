//! Participant implementation
use std::{convert::TryInto, sync::Arc};

use async_trait::async_trait;
use futures::future::FutureExt;
use thiserror::Error;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
};
use tracing::warn;

use modalic_core::mask::Model;

use crate::{
    client::{
        new_client,
        settings::{Settings, SettingsError},
        ClientError,
    },
    http_client::HttpClient,
    LocalModelConfig,
    ModelStore,
    Notify,
    SerializableState,
    StateMachine,
    TransitionOutcome,
    MosaicClientTrait,
};

/// Event emitted by the participant internal state machine as it advances through the
/// PET protocol
pub enum Event {
    /// Event emitted when the participant is selected for the update task
    Update,
    /// Event emitted when the participant is selected for the sum task
    Sum,
    /// Event emitted when the participant is done with its task
    Idle,
    /// Event emitted when a new round starts
    NewRound,
    /// Event emitted when the participant should load its model. This only happens if
    /// the participant has been selected for the update task
    LoadModel,
}

/// Event sender that is passed to the participant internal state machine for emitting
/// notification
pub struct Notifier(mpsc::Sender<Event>);
impl Notifier {
    fn notify(&mut self, event: Event) {
        if let Err(e) = self.0.try_send(event) {
            warn!("failed to notify participant: {}", e);
        }
    }
}

/// A receiver for events emitted by the participant internal state machine
pub struct Events(mpsc::Receiver<Event>);

impl Events {
    /// Create a new event sender and receiver.
    fn new() -> (Self, Notifier) {
        let (tx, rx) = mpsc::channel(10);
        (Self(rx), Notifier(tx))
    }

    /// Pop the next event. If no event has been received, return `None`.
    fn next(&mut self) -> Option<Event> {
        // Note `try_recv` (tokio 0.2.x) or `recv().now_or_never()` (tokio 1.x)
        // has an implementation bug where previously sent messages may not be
        // available immediately.
        // Related issue: https://github.com/tokio-rs/tokio/issues/3350
        // However, that should not be an issue for us.
        let next = self.0.recv().now_or_never()?;
        if next.is_none() {
            // if next is `none`, the channel is closed
            // This can happen if:
            //  1. the state machine crashed. In that case it's OK to crash.
            //  2. `next` was called whereas the state machine was
            //     dropped, which is an error. So crashing is OK as
            //     well.
            panic!("notifier dropped")
        }
        next
    }
}

impl Notify for Notifier {
    fn new_round(&mut self) {
        self.notify(Event::NewRound)
    }
    fn sum(&mut self) {
        self.notify(Event::Sum)
    }
    fn update(&mut self) {
        self.notify(Event::Update)
    }
    fn load_model(&mut self) {
        self.notify(Event::LoadModel)
    }
    fn idle(&mut self) {
        self.notify(Event::Idle)
    }
}

/// A store shared between by the participant and its internal state machine. When the
/// state machine emits a [`Event::LoadModel`] event, the participant is expected to
/// load its model into the store. See [`Participant::set_model()`].
#[derive(Clone)]
struct Store(Arc<Mutex<Option<Model>>>);

impl Store {
    /// Create a new model store.
    fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

#[async_trait]
impl ModelStore for Store {
    type Model = Model;
    type Error = std::convert::Infallible;

    async fn load_model(&mut self) -> Result<Option<Self::Model>, Self::Error> {
        Ok(self.0.lock().await.take())
    }
}

/// Represent the participant current task
#[derive(Clone, Debug, Copy)]
pub enum Task {
    /// The participant is taking part in the sum task
    Sum,
    /// The participant is taking part in the update task
    Update,
    /// The participant is not taking part in any task
    None,
}

/// A participant. It embeds an internal state machine that executes the PET
/// protocol. However, it is the caller's responsibility to drive this state machine by
/// calling [`Participant::tick()`], and to take action when the participant state
/// changes.
pub struct Client {
    /// Internal state machine
    state_machine: Option<StateMachine>,
    /// Receiver for the events emitted by the state machine
    events: Events,
    /// Model store where the participant should load its model, when
    /// `self.should_set_model` is `true`.
    store: Store,
    /// Async runtime to execute the state machine
    runtime: Runtime,
    /// Xaynet client
    http_client: HttpClient<reqwest::Client>,
    /// Whether the participant state changed after the last call to
    /// [`Participant::tick()`]
    made_progress: bool,
    /// Whether the participant should load its model into the store.
    should_set_model: bool,
    /// Whether a new global model is available.
    new_global_model: bool,
    /// The participant current task
    task: Task,
}

/// Error that can occur when instantiating a new [`Participant`], either with
/// [`Participant::new()`] or [`Participant::restore()`]
#[derive(Error, Debug)]
pub enum InitError {
    #[error("failed to deserialize the participant state {:?}", _0)]
    Deserialization(#[from] Box<bincode::ErrorKind>),
    #[error("failed to initialize the participant runtime {:?}", _0)]
    Runtime(std::io::Error),
    #[error("failed to initialize HTTP client {:?}", _0)]
    Client(#[from] ClientError),
    #[error("invalid participant settings {:?}", _0)]
    InvalidSettings(#[from] SettingsError),
}

#[derive(Error, Debug)]
#[error("failed to fetch global model: {}", self.0)]
pub struct GetGlobalModelError(crate::http_client::ClientError);

impl Client {
    /// Create a new participant with the given settings
    pub fn new(settings: Settings) -> Result<Self, InitError> {
        let (url, pet_settings) = settings.try_into()?;
        let client = new_client(url.as_str(), None, None)?;
        let (events, notifier) = Events::new();
        let store = Store::new();
        let state_machine =
            StateMachine::new(pet_settings, client.clone(), store.clone(), notifier);
        Self::init(state_machine, client, events, store)
    }

    /// Restore a participant from it's serialized state. The coordinator client that
    /// the participant uses internally is not part of the participant state, so the
    /// `url` is used to instantiate a new one.
    pub fn restore(state: &[u8], url: &str) -> Result<Self, InitError> {
        let state: SerializableState = bincode::deserialize(state)?;
        let (events, notifier) = Events::new();
        let store = Store::new();
        let client = new_client(url, None, None)?;
        let state_machine = StateMachine::restore(state, client.clone(), store.clone(), notifier);
        Self::init(state_machine, client, events, store)
    }

    fn init(
        state_machine: StateMachine,
        http_client: HttpClient<reqwest::Client>,
        events: Events,
        store: Store,
    ) -> Result<Self, InitError> {
        let mut client = Self {
            runtime: Self::runtime()?,
            state_machine: Some(state_machine),
            events,
            store,
            http_client,
            task: Task::None,
            made_progress: true,
            should_set_model: false,
            new_global_model: false,
        };
        client.process_events();
        Ok(client)
    }

    fn runtime() -> Result<Runtime, InitError> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(InitError::Runtime)
    }

    /// Serialize the participant state and return the corresponding buffer.
    pub fn save(self) -> Vec<u8> {
        // UNWRAP_SAFE: the state machine is always set.
        let state_machine = self.state_machine.unwrap().save();
        bincode::serialize(&state_machine).unwrap()
    }

    /// Drive the participant internal state machine.
    ///
    /// After calling this method, the caller should check whether the participant state
    /// changed, by calling [`Participant::made_progress()`].  If the state changed, the
    /// caller should perform the following checks and react appropriately:
    ///
    /// - whether the participant is taking part to any task by calling
    ///   [`Participant::task()`]
    /// - whether the participant should load its model into the store by calling
    ///   [`Participant::should_set_model()`]
    pub fn tick(&mut self) {
        println!("\tParticipant tick!");
        // UNWRAP_SAFE: the state machine is always set.
        let state_machine = self.state_machine.take().unwrap();
        let outcome = self
            .runtime
            .block_on(async { state_machine.transition().await });
        println!("\tParticipant tick : {:?}", &outcome);
        match outcome {
            TransitionOutcome::Pending(new_state_machine) => {
                self.made_progress = false;
                self.state_machine = Some(new_state_machine);
            }
            TransitionOutcome::Complete(new_state_machine) => {
                self.made_progress = true;
                self.state_machine = Some(new_state_machine)
            }
        };
        self.process_events();
    }

    fn process_events(&mut self) {
        println!("\tParticipant process event");
        
        loop {
            match self.events.next() {
                Some(Event::Idle) => {
                    self.task = Task::None;
                    println!("\tParticipant process event : {:?}", &self.task);
                }
                Some(Event::Update) => {
                    self.task = Task::Update;
                    println!("\tParticipant process event : {:?}", &self.task);
                }
                Some(Event::Sum) => {
                    self.task = Task::Sum;
                    println!("\tParticipant process event : {:?}", &self.task);
                }
                Some(Event::NewRound) => {
                    self.should_set_model = false;
                    self.new_global_model = true;
                    println!("\tParticipant process event : {:?}", &self.task);
                }
                Some(Event::LoadModel) => {
                    self.should_set_model = true;
                    println!("\tParticipant process event : {:?}", &self.task);
                }
                None => {
                    println!("\tParticipant process event : {:?}", &self.task);
                    break;
                }

            }
        }
    }

    /// Check whether the participant internal state machine made progress while
    /// executing the PET protocol. If so, the participant state likely changed.
    pub fn made_progress(&self) -> bool {
        self.made_progress
    }

    /// Check whether the participant internal state machine is waiting for the
    /// participant to load its model into the store. If this method returns `true`, the
    /// caller should make sure to call [`Participant::set_model()`] at some point.
    pub fn should_set_model(&self) -> bool {
        self.should_set_model
    }

    /// Check whether a new global model is available. If this method returns `true`, the
    /// caller can call [`Participant::global_model()`] to fetch the new global model.
    pub fn new_global_model(&self) -> bool {
        self.new_global_model
    }

    /// Return the participant current task
    pub fn task(&self) -> Task {
        self.task
    }

    /// Load the given model into the store, so that the participant internal state
    /// machine can process it.
    pub fn set_model(&mut self, model: Model) {
        let Self {
            ref mut runtime,
            ref store,
            ..
        } = self;

        println!("before runtime.");

        runtime.block_on(async {
            let mut stored_model = store.0.lock().await;
            *stored_model = Some(model)
        });

        // println!("I gu")
        self.should_set_model = false;
        println!("I set the model now what?");
    }

    /// Retrieve the current global model, if available.
    pub fn global_model(&mut self) -> Result<Option<Model>, GetGlobalModelError> {
        let Self {
            ref mut runtime,
            ref mut http_client,
            ..
        } = self;

        let global_model =
            runtime.block_on(async { http_client.get_model().await.map_err(GetGlobalModelError) });
        if global_model.is_ok() {
            self.new_global_model = false;
        }
        global_model
    }

    /// Return the local model configuration of the model that is expected in the
    /// [`Participant::set_model`] method.
    pub fn local_model_config(&self) -> LocalModelConfig {
        // UNWRAP_SAFE: the state machine is always set.
        let state_machine = self.state_machine.as_ref().unwrap();
        state_machine.local_model_config()
    }
}
