use derive_more::From;
use futures::Stream;
use std::io::{Error, ErrorKind};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::{mpsc, oneshot};

use crate::message::Message;

/// A handle to send requests to the [`Engine`].
#[derive(Clone, From, Debug)]
pub struct RequestSender(pub mpsc::UnboundedSender<Message>);

impl RequestSender {
    pub fn new() -> (RequestReceiver, RequestSender) {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        (RequestReceiver(rx), RequestSender(tx))
    }
    pub async fn send(&mut self, req: Message) -> Result<(), Error> {
        let (_tx, rx) = oneshot::channel::<Result<(), Error>>();
        self.0.send(req).map_err(|_| {
            Error::new(
                ErrorKind::Other,
                "failed to send request to the engine: engine shuts down.",
            )
        })?;
        rx.await.map_err(|_| {
            Error::new(
                ErrorKind::Other,
                "failed to receive response from the engine.",
            )
        })?
    }
}

/// A handle to receive requests that the ['Engine'] makes use of.
#[derive(From, Debug)]
pub struct RequestReceiver(mpsc::UnboundedReceiver<Message>);

impl Stream for RequestReceiver {
    type Item = Message;
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
    /// Closes the `Request` channel.
    pub fn close(&mut self) {
        self.0.close()
    }
}
