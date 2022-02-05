//! Module for handling and processing certain changes and updates that are produced by the engine
//! to the clients.
use tokio::sync::watch;

use crate::core::model::ModelUpdate;

#[derive(Debug)]
pub struct Publisher {
    tx: Broadcast,
}

impl Publisher {
    pub fn new(model: ModelUpdate) -> (Publisher, Subscriber) {
        let (model_tx, model_rx) = watch::channel::<ModelUpdate>(model);
        let publisher = Publisher {
            tx: Broadcast(model_tx),
        };
        let subscriber = Subscriber {
            rx: Listener(model_rx),
        };
        (publisher, subscriber)
    }

    /// broadcasting the updated global model.
    pub fn broadcast_model(&mut self, model: ModelUpdate) {
        let _ = self.tx.0.send(model);
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub rx: Listener,
}

/// A watch channel to send events to clients.
#[derive(Debug)]
pub struct Broadcast(watch::Sender<ModelUpdate>);

/// A watch channel that functions as a listener.
#[derive(Debug, Clone)]
pub struct Listener(watch::Receiver<ModelUpdate>);

impl From<watch::Receiver<ModelUpdate>> for Listener {
    fn from(receiver: watch::Receiver<ModelUpdate>) -> Self {
        Listener(receiver)
    }
}

impl Listener {
    pub fn recv(&self) -> ModelUpdate {
        self.0.borrow().clone()
    }
}
