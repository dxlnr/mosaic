//! Module for handling and processing certain changes and updates that are produced by the engine
//! to the clients.
use std::sync::Arc;
use tokio::sync::watch;

use crate::engine::model::ModelUpdate;

#[derive(Debug)]
pub struct Publisher {
    tx: Broadcast,
}

impl Publisher {
    pub fn new(model: ModelUpdate) -> (Publisher, Subscriber) {
        let (model_tx, model_rx) = watch::channel::<Arc<ModelUpdate>>(Arc::new(model));
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
        let _ = self.tx.0.send(Arc::new(model));
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub rx: Listener,
}

/// A watch channel to send events to clients.
#[derive(Debug)]
pub struct Broadcast(watch::Sender<Arc<ModelUpdate>>);

/// A watch channel that functions as a listener.
#[derive(Debug, Clone)]
pub struct Listener(watch::Receiver<Arc<ModelUpdate>>);

impl From<watch::Receiver<Arc<ModelUpdate>>> for Listener {
    fn from(receiver: watch::Receiver<Arc<ModelUpdate>>) -> Self {
        Listener(receiver)
    }
}

impl Listener {
    pub fn recv(&self) -> Arc<ModelUpdate> {
        self.0.borrow().clone()
    }
}
