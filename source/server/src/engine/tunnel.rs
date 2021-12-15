use derive_more::From;
use futures::{future::FutureExt, Stream};
use std::convert::Infallible;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::{mpsc, oneshot};

/// A handle to send requests to the [`Engine`].
#[derive(Clone, From, Debug)]
pub struct RequestSender(mpsc::UnboundedSender<EngineRequest>);

impl RequestSender {
    pub fn new() -> (RequestReceiver, RequestSender) {
        let (tx, rx) = mpsc::unbounded_channel::<EngineRequest>();
        (RequestReceiver(rx), RequestSender(tx))
    }
    pub async fn sending(&mut self, req: EngineRequest) -> Result<(), Infallible> {
        todo!()
    }
}

/// A handle to receive requests that the ['Engine'] makes use of.
#[derive(From, Debug)]
pub struct RequestReceiver(mpsc::UnboundedReceiver<EngineRequest>);

impl Stream for RequestReceiver {
    type Item = EngineRequest;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().0).poll_recv(cx)
    }
}

pub struct EngineRequest {
    pub model: Vec<Vec<u8>>,
}

struct Inner {}
struct Shared {}

impl RequestReceiver {
    pub fn recv(&mut self) {
        todo!()
    }

    pub fn try_recv(&mut self) -> Option<()> {
        todo!()
    }
}

// pub fn channel() (RequestSender, RequestReceiver) {}
