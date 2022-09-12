//! Module for publishing & subcribing to certain events when processing the [`StateEngine`].
//! 
//! Uses the [tokio watch](https://docs.rs/tokio/latest/tokio/sync/watch/index.html): 
//! A single-producer, multi-consumer channel that only retains the last sent value.
//!
use tokio::sync::watch;

/// An [`Event`] object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<E> {
    /// The event itself
    pub event: E,
}

/// The [`EventPublisher`] for broadcasting events.
#[derive(Debug)]
pub struct EventPublisher {}

impl EventPublisher {
    pub fn new() -> (EventPublisher, EventSubscriber) {
        todo!()
    }
    /// prepares and provides an event object E.
    fn event<E>(&self, event: E) -> Event<E> {
        Event { event }
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