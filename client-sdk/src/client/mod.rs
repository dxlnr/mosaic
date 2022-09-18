use thiserror::Error;
use tokio::{
    runtime::Runtime,
    sync::mpsc,
};

use crate::state_engine::StateEngine;

pub enum Event {
    /// Event emitted when the client is done with its task.
    Idle,
}

/// A receiver for events emitted by the clients internal [`StateEngine`].
pub struct EventSubscriber(mpsc::Receiver<Event>);

/// Storage API
/// 
#[derive(Clone)]
struct Store {}


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
}

/// [`Client`]
///  
/// The client holds an internal [`StateEngine`] that executes the FL protocol. 
/// 
pub struct Client {
    /// Internal [`StateEngine`] of the client.
    engine: StateEngine,
    /// Receiver for the events emitted by the [`StateEngine`].
    events: EventSubscriber,
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
}