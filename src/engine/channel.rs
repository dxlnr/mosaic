//! Module creating a channel for transfering messages between client and engine.
//!
use derive_more::From;
use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::{mpsc, oneshot};

use crate::{proxy::message::Message, service::error::ServiceError};

#[derive(Clone, From, Debug)]
/// A handle to send requests to the ['Engine'].
pub struct RequestSender(pub mpsc::UnboundedSender<(Message, ResponseSender)>);

impl RequestSender {
    pub fn new() -> (RequestReceiver, RequestSender) {
        let (tx, rx) = mpsc::unbounded_channel::<(Message, ResponseSender)>();
        (RequestReceiver(rx), RequestSender(tx))
    }
    pub async fn send(&mut self, req: Message) -> Result<(), ServiceError> {
        let (tx, rx) = oneshot::channel::<Result<(), ServiceError>>();
        self.0
            .send((req, tx))
            .map_err(|_| ServiceError::RequestError)?;
        rx.await.map_err(|_| ServiceError::RequestError)?
    }
}

/// A handle to send a response upon the request receiver.
pub type ResponseSender = oneshot::Sender<Result<(), ServiceError>>;
#[derive(From, Debug)]
/// A handle to receive requests that the ['Engine'] makes use of.
pub struct RequestReceiver(mpsc::UnboundedReceiver<(Message, ResponseSender)>);

impl Stream for RequestReceiver {
    type Item = (Message, ResponseSender);
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