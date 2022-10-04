//! Communication channels for sending messages between client and the aggregators [`StateEngine`].
//!
//! Uses the [tokio mspc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) module:
//! A multi-producer, single-consumer queue for sending values across asynchronous tasks.
//!
use derive_more::{Display, From};
use futures::{future::FutureExt, Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use mosaic_core::message::grpc::mosaic::protos::client_message::ClientUpdate;
// use crate::grpc::msflp::client_message::ClientUpdate;
// use msflp::client_message::ClientUpdate;

use mosaic_core::{model::Model, crypto::{PublicSigningKey, ByteObject}, mask::MaskSeed};

/// Errors which can occur while the state engine handles a request.
#[derive(Debug, Display, Error)]
pub enum RequestError {
    /// The request could not be processed due to an internal error: {0}.
    InternalError(&'static str),
    /// The message was rejected.
    MessageRejected,
    /// The request will be discarded due to an error: {0}.
    RequestDiscarded(&'static str),

}

#[derive(Debug)]
pub struct StateEngineRequest {
    /// The Client Identifier.
    pub client_id: Option<u32>,
    /// The public key of the client.
    pub client_pk: Option<PublicSigningKey>,
    /// The local seed defines the seed used to mask `masked_model`.
    pub local_seed: Option<MaskSeed>,
    /// The masked model trained by the participant.
    pub model: Model,
}

impl From<ClientUpdate> for StateEngineRequest {
    fn from(client_update: ClientUpdate) -> StateEngineRequest  {
        let c_pk = PublicSigningKey::from_slice(&client_update.client_pk);
        let m_seed = MaskSeed::from_slice(&client_update.local_seed);
        let c_model = Model::from_proto(client_update.model, client_update.model_version);
        StateEngineRequest { client_id: Some(client_update.client_id), client_pk: c_pk, local_seed: m_seed, model: c_model}
    }
}


#[derive(Clone, From, Debug)]
/// A handle to send requests to the ['StateEngine'].
pub struct RequestSender(pub mpsc::UnboundedSender<(StateEngineRequest, ResponseSender)>);

impl RequestSender {
    pub fn new() -> (RequestReceiver, RequestSender) {
        let (tx, rx) = mpsc::unbounded_channel::<(StateEngineRequest, ResponseSender)>();
        (RequestReceiver(rx), RequestSender(tx))
    }
    pub async fn send(&mut self, req: StateEngineRequest) -> Result<(), RequestError> {
        let (tx, rx) = oneshot::channel::<Result<(), RequestError>>();
        self.0.send((req, tx)).map_err(|_| {
            RequestError::InternalError(
                "failed to send request to the state engine: state engine is down.",
            )
        })?;
        rx.await.map_err(|_| {
            RequestError::InternalError("Unable to receive a response from the state engine.")
        })?
    }
}

/// A handle to send a response upon the request receiver.
pub type ResponseSender = oneshot::Sender<Result<(), RequestError>>;
#[derive(From, Debug)]
/// A handle to receive requests that the ['StateEngine'] makes use of.
pub struct RequestReceiver(mpsc::UnboundedReceiver<(StateEngineRequest, ResponseSender)>);

impl Stream for RequestReceiver {
    type Item = (StateEngineRequest, ResponseSender);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_recv(cx)
    }
}

impl RequestReceiver {
    /// Receives the next request.
    /// See [the `tokio` documentation][receive] for more information.
    ///
    /// [receive]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html#method.recv
    pub async fn recv(&mut self) -> Option<(StateEngineRequest, ResponseSender)> {
        self.0.recv().await
    }

    /// Try to retrieve the next request without blocking.
    /// 
    pub fn try_recv(&mut self) -> Option<Option<(StateEngineRequest, ResponseSender)>> {
        self.0.recv().now_or_never()
    }

    /// Closes the [`RequestReceiver`] channel.
    /// See [the `tokio` documentation][close] for more information.
    ///
    /// [close]: https://docs.rs/tokio/latest/tokio/sync/mpsc/struct.UnboundedReceiver.html#method.close
    pub fn close(&mut self) {
        self.0.close()
    }
}
