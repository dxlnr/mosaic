//! Module for handling and processing certain changes and updates that are produced by the engine
//! to the clients.
use tokio::sync::watch;

use crate::{core::model::ModelUpdate, rest::stats::StatsUpdate};

/// An event emitted by the coordinator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<E> {
    // /// Potential metadata that is associated to this event
    // meta: u64,
    /// The event itself
    pub event: E,
}

#[derive(Debug)]
pub struct Publisher {
    tx_model: Broadcast<ModelUpdate>,
    tx_stats: Broadcast<StatsUpdate>,
}

impl Publisher {
    pub fn new(model: ModelUpdate, stats: StatsUpdate) -> (Publisher, Subscriber) {
        let (model_tx, model_rx) = watch::channel::<Event<ModelUpdate>>(Event { event: model });
        let (stats_tx, stats_rx) = watch::channel::<Event<StatsUpdate>>(Event { event: stats });
        let publisher = Publisher {
            tx_model: Broadcast(model_tx),
            tx_stats: Broadcast(stats_tx),
        };
        let subscriber = Subscriber {
            rx_model: Listener(model_rx),
            rx_stats: Listener(stats_rx),
        };
        (publisher, subscriber)
    }
    /// prepares and provides an event object E.
    fn event<E>(&self, event: E) -> Event<E> {
        Event { event }
    }

    /// broadcasting the updated global model.
    pub fn broadcast_model(&mut self, model: ModelUpdate) {
        let _ = self.tx_model.0.send(self.event(model));
    }

    /// broadcasting metrics for the running process.
    pub fn broadcast_stats(&mut self, stats: StatsUpdate) {
        let _ = self.tx_stats.0.send(self.event(stats));
    }
}

/// The [`Subscriber`] holds event listeners for every generic purpose.
#[derive(Debug, Clone)]
pub struct Subscriber {
    pub rx_model: Listener<ModelUpdate>,
    pub rx_stats: Listener<StatsUpdate>,
}

impl Subscriber {
    /// Get a listener for new model events
    pub fn get_listener_model(&self) -> Listener<ModelUpdate> {
        self.rx_model.clone()
    }

    pub fn get_listener_stats(&self) -> Listener<StatsUpdate> {
        self.rx_stats.clone()
    }
}

#[derive(Debug)]
/// A watch channel to send events to clients.
pub struct Broadcast<E>(watch::Sender<Event<E>>);

#[derive(Debug, Clone)]
/// A watch channel that functions as a listener.
pub struct Listener<E>(watch::Receiver<Event<E>>);

impl<E> From<watch::Receiver<Event<E>>> for Listener<E> {
    fn from(receiver: watch::Receiver<Event<E>>) -> Self {
        Listener(receiver)
    }
}

impl<E> Listener<E>
where
    E: Clone,
{
    pub fn recv(&self) -> Event<E> {
        self.0.borrow().clone()
    }
}
