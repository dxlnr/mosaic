//! Module for publishing & subcribing to certain events when processing the [`StateEngine`].
//!
//! Uses the [tokio watch](https://docs.rs/tokio/latest/tokio/sync/watch/index.html):
//! A single-producer, multi-consumer channel that only retains the last sent value.
//!
use tokio::sync::watch;

use crate::state_engine::states::StateName;

/// An [`Event`] object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<E> {
    /// Metadata that associates this event to the round in which it is
    /// emitted.
    pub round_id: u32,
    /// The event itself
    pub event: E,
}

/// The [`EventPublisher`] for broadcasting events.
#[derive(Debug)]
pub struct EventPublisher {
    round_id: u32,
    tx_state: EventBroadcast<StateName>
}

impl EventPublisher {
    pub fn new(round_id: u32, state: StateName) -> (EventPublisher, EventSubscriber) {
        
        let (tx_state, rx_state) = watch::channel::<Event<StateName>>(Event {
            round_id,
            event: state,
        });
        // let (model_tx, model_rx) = watch::channel::<Event<ModelUpdate>>(Event { event: model });
        // let (stats_tx, stats_rx) = watch::channel::<Event<StatsUpdate>>(Event { event: stats });
        
        let publisher = EventPublisher {
            round_id,
            tx_state: tx_state.into(),
        };
        let subscriber = EventSubscriber {};

        (publisher, subscriber)
    }

    /// prepares and provides an event object E.
    fn event<E>(&self, event: E) -> Event<E> {
        Event { round_id: self.round_id, event }
    }

    /// Emit a phase event
    pub fn publish_state(&mut self, state: StateName) {
        let _ = self.tx_state.broadcast(self.event(state));
    }
}

/// The [`EventSubscriber`] holds event listeners for every generic purpose.
#[derive(Debug, Clone)]
pub struct EventSubscriber {}

// impl EventSubscriber {
//     /// Get a listener for new model events
//     pub fn get_listener_model(&self) -> EventListener<ModelUpdate> {
//         self.rx_model.clone()
//     }

//     pub fn get_listener_stats(&self) -> EventListener<StatsUpdate> {
//         self.rx_stats.clone()
//     }
// }

#[derive(Debug)]
/// A watch channel to send events to clients.
pub struct EventBroadcast<E>(watch::Sender<Event<E>>);

impl<E> EventBroadcast<E> {
    /// Send `event` to all the `EventListener<E>`
    fn broadcast(&self, event: Event<E>) {
        // We don't care whether there's a listener or not
        let _ = self.0.send(event);
    }
}

impl<E> From<watch::Sender<Event<E>>> for EventBroadcast<E> {
    fn from(sender: watch::Sender<Event<E>>) -> Self {
        Self(sender)
    }
}

#[derive(Debug, Clone)]
/// A watch channel that functions as a listener.
pub struct EventListener<E>(watch::Receiver<Event<E>>);

impl<E> From<watch::Receiver<Event<E>>> for EventListener<E> {
    fn from(receiver: watch::Receiver<Event<E>>) -> Self {
        EventListener(receiver)
    }
}

impl<E> EventListener<E>
where
    E: Clone,
{
    pub fn recv(&self) -> Event<E> {
        self.0.borrow().clone()
    }
}
