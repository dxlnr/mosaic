//! Communication channels for sending messages between client and the aggregators [`StateEngine`].
//!
//! Uses the [tokio mspc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html) module:
//! A multi-producer, single-consumer queue for sending values across asynchronous tasks.
//!

use derive_more::{Display, From};
use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

/// Errors which can occur while the state engine handles a request.
#[derive(Debug, Display, Error)]
pub enum RequestError {
    /// The request could not be processed due to an internal error: {0}.
    InternalError(&'static str),
}

#[derive(Debug)]
pub struct StateEngineRequest {}

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
        self.0
            .send((req, tx))
            .map_err(|_| RequestError::InternalError(
                "failed to send request to the state engine: state engine is down.",
            ))?;
        rx.await.map_err(|_| RequestError::InternalError("Unable to receive a response from the state engine."))?
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
    pub fn recv(&mut self) {
        todo!()
    }

    pub fn try_recv(&mut self) -> Option<()> {
        todo!()
    }
    /// Closes the [`RequestReceiver`] channel.
    pub fn close(&mut self) {
        self.0.close()
    }
}
