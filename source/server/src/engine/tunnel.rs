use derive_more::From;
use std::convert::Infallible;
use tokio::sync::{mpsc, oneshot};

/// A handle to send requests to the [`Engine`].
///
#[derive(Clone, From, Debug)]
pub struct RequestSender(mpsc::UnboundedSender<()>);

impl RequestSender {
    pub fn new() -> (RequestReceiver, RequestSender) {
        let (tx, rx) = mpsc::unbounded_channel::<()>();
        (RequestReceiver::from(rx), RequestSender::from(tx))
    }
    pub async fn sending(&mut self, req: EngineRequest) -> Result<(), Infallible> {
        todo!()
    }
}

/// A handle to receive requests that the ['Engine'] makes use of.
///
#[derive(From, Debug)]
pub struct RequestReceiver(mpsc::UnboundedReceiver<()>);

pub struct EngineRequest;
struct Inner {}
struct Shared {}

impl RequestReceiver {
    pub fn recv(&mut self) {
        todo!()
    }

    pub fn try_recv(&mut self) -> Option<()> {
        todo!()
    }
    // /// Closes the `Request` channel.
    // /// Check [`tokio` documentation][close] for more information.
    // pub fn close(&mut self) {
    //     self.0.close()
    // }
}

// pub fn channel() (RequestSender, RequestReceiver) {}
