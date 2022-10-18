//! This module provides the `StateMachine`, `Events`, `EventSubscriber` and `EventPublisher` types.

use std::sync::Arc;

use tokio::sync::watch;

use crate::state_engine::states::StateName;
use mosaic_core::{
    common::RoundParameters, crypto::EncryptKeyPair, model::Model, SeedDict, SumDict,
};

/// An event emitted by the coordinator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<E> {
    /// Metadata that associates this event to the round in which it is
    /// emitted.
    pub round_id: u32,
    /// The event itself
    pub event: E,
}

// FIXME: should we simply use `Option`s here?
/// Global model update event.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelUpdate {
    Invalidate,
    New(Arc<Model>),
}

/// Dictionary update event.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DictionaryUpdate<D> {
    Invalidate,
    New(Arc<D>),
}

/// A convenience type to emit any coordinator event.
#[derive(Debug)]
pub struct EventPublisher {
    /// Round ID that is attached to all the requests.
    round_id: u32,
    keys_tx: EventBroadcaster<EncryptKeyPair>,
    params_tx: EventBroadcaster<RoundParameters>,
    state_tx: EventBroadcaster<StateName>,
    model_tx: EventBroadcaster<ModelUpdate>,
    sum_dict_tx: EventBroadcaster<DictionaryUpdate<SumDict>>,
    seed_dict_tx: EventBroadcaster<DictionaryUpdate<SeedDict>>,
}

/// The `EventSubscriber` hands out `EventListener`s for any
/// coordinator event.
#[derive(Debug)]
pub struct EventSubscriber {
    keys_rx: EventListener<EncryptKeyPair>,
    params_rx: EventListener<RoundParameters>,
    state_rx: EventListener<StateName>,
    model_rx: EventListener<ModelUpdate>,
    sum_dict_rx: EventListener<DictionaryUpdate<SumDict>>,
    seed_dict_rx: EventListener<DictionaryUpdate<SeedDict>>,
}

impl EventPublisher {
    /// Initialize a new event publisher with the given initial events.
    pub fn init(
        round_id: u32,
        keys: EncryptKeyPair,
        params: RoundParameters,
        state: StateName,
        model: ModelUpdate,
    ) -> (Self, EventSubscriber) {
        let (keys_tx, keys_rx) = watch::channel::<Event<EncryptKeyPair>>(Event {
            round_id,
            event: keys,
        });

        let (params_tx, params_rx) = watch::channel::<Event<RoundParameters>>(Event {
            round_id,
            event: params,
        });

        let (state_tx, state_rx) = watch::channel::<Event<StateName>>(Event {
            round_id,
            event: state,
        });

        let (model_tx, model_rx) = watch::channel::<Event<ModelUpdate>>(Event {
            round_id,
            event: model,
        });

        let (sum_dict_tx, sum_dict_rx) =
            watch::channel::<Event<DictionaryUpdate<SumDict>>>(Event {
                round_id,
                event: DictionaryUpdate::Invalidate,
            });

        let (seed_dict_tx, seed_dict_rx) =
            watch::channel::<Event<DictionaryUpdate<SeedDict>>>(Event {
                round_id,
                event: DictionaryUpdate::Invalidate,
            });

        let publisher = EventPublisher {
            round_id,
            keys_tx: keys_tx.into(),
            params_tx: params_tx.into(),
            state_tx: state_tx.into(),
            model_tx: model_tx.into(),
            sum_dict_tx: sum_dict_tx.into(),
            seed_dict_tx: seed_dict_tx.into(),
        };

        let subscriber = EventSubscriber {
            keys_rx: keys_rx.into(),
            params_rx: params_rx.into(),
            state_rx: state_rx.into(),
            model_rx: model_rx.into(),
            sum_dict_rx: sum_dict_rx.into(),
            seed_dict_rx: seed_dict_rx.into(),
        };

        (publisher, subscriber)
    }

    /// Set the round ID that is attached to the events the publisher broadcasts.
    pub fn set_round_id(&mut self, id: u32) {
        self.round_id = id;
    }

    fn event<T>(&self, event: T) -> Event<T> {
        Event {
            round_id: self.round_id,
            event,
        }
    }

    /// Emit a keys event
    pub fn broadcast_keys(&mut self, keys: EncryptKeyPair) {
        let _ = self.keys_tx.broadcast(self.event(keys));
    }

    /// Emit a round parameters event
    pub fn broadcast_params(&mut self, params: RoundParameters) {
        let _ = self.params_tx.broadcast(self.event(params));
    }

    /// Emit a state event
    pub fn broadcast_state(&mut self, state: StateName) {
        let _ = self.state_tx.broadcast(self.event(state));
    }

    /// Emit a model event
    pub fn broadcast_model(&mut self, update: ModelUpdate) {
        let _ = self.model_tx.broadcast(self.event(update));
    }

    /// Emit a sum dictionary update
    pub fn broadcast_sum_dict(&mut self, update: DictionaryUpdate<SumDict>) {
        let _ = self.sum_dict_tx.broadcast(self.event(update));
    }

    /// Emit a seed dictionary update
    pub fn broadcast_seed_dict(&mut self, update: DictionaryUpdate<SeedDict>) {
        let _ = self.seed_dict_tx.broadcast(self.event(update));
    }
}

impl EventSubscriber {
    /// Get a listener for keys events. Callers must be careful not to
    /// leak the secret key they receive, since that would compromise
    /// the security of the coordinator.
    pub fn keys_listener(&self) -> EventListener<EncryptKeyPair> {
        self.keys_rx.clone()
    }
    /// Get a listener for round parameters events
    pub fn params_listener(&self) -> EventListener<RoundParameters> {
        self.params_rx.clone()
    }

    /// Get a listener for new state events
    pub fn state_listener(&self) -> EventListener<StateName> {
        self.state_rx.clone()
    }

    /// Get a listener for new model events
    pub fn model_listener(&self) -> EventListener<ModelUpdate> {
        self.model_rx.clone()
    }

    /// Get a listener for sum dictionary updates
    pub fn sum_dict_listener(&self) -> EventListener<DictionaryUpdate<SumDict>> {
        self.sum_dict_rx.clone()
    }

    /// Get a listener for seed dictionary updates
    pub fn seed_dict_listener(&self) -> EventListener<DictionaryUpdate<SeedDict>> {
        self.seed_dict_rx.clone()
    }
}

/// A listener for coordinator events. It can be used to either
/// retrieve the latest `Event<E>` emitted by the coordinator (with
/// `EventListener::get_latest`).
#[derive(Debug, Clone)]
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
    pub fn get_latest(&self) -> Event<E> {
        self.0.borrow().clone()
    }

    #[cfg(test)]
    pub async fn changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.0.changed().await
    }
}

/// A channel to send `Event<E>` to all the `EventListener<E>`.
#[derive(Debug)]
pub struct EventBroadcaster<E>(watch::Sender<Event<E>>);

impl<E> EventBroadcaster<E> {
    /// Send `event` to all the `EventListener<E>`
    fn broadcast(&self, event: Event<E>) {
        // We don't care whether there's a listener or not
        let _ = self.0.send(event);
    }
}

impl<E> From<watch::Sender<Event<E>>> for EventBroadcaster<E> {
    fn from(sender: watch::Sender<Event<E>>) -> Self {
        Self(sender)
    }
}
