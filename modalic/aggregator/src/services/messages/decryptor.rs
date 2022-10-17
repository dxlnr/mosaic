use std::{pin::Pin, sync::Arc, task::Poll};

use futures::{future::Future, task::Context};
use rayon::ThreadPool;
use tokio::sync::oneshot;
use tower::{
    limit::concurrency::{future::ResponseFuture, ConcurrencyLimit},
    Service,
};
use tracing::{debug, trace};

use crate::{
    services::messages::{BoxedServiceFuture, ServiceError},
    state_engine::events::{EventListener, EventSubscriber},
};
use modalic_core::crypto::EncryptKeyPair;

/// A service for decrypting PET messages.
///
/// Since this is a CPU-intensive task for large messages, this
/// service offloads the processing to a `rayon` thread-pool to avoid
/// overloading the tokio thread-pool with blocking tasks.
#[derive(Clone)]
struct RawDecryptor {
    /// A listener to retrieve the latest coordinator keys. These are
    /// necessary for decrypting messages and verifying their
    /// signature.
    keys_events: EventListener<EncryptKeyPair>,

    /// Thread-pool the CPU-intensive tasks are offloaded to.
    thread_pool: Arc<ThreadPool>,
}

impl<T> Service<T> for RawDecryptor
where
    T: AsRef<[u8]> + Sync + Send + 'static,
{
    type Response = Vec<u8>;
    type Error = ServiceError;
    #[allow(clippy::type_complexity)]
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, data: T) -> Self::Future {
        debug!("retrieving the current keys");
        let keys = self.keys_events.get_latest().event;
        let (tx, rx) = oneshot::channel::<Result<Self::Response, Self::Error>>();

        trace!("spawning decryption task on threadpool");
        self.thread_pool.spawn(move || {
            debug!("decrypting message");
            let res = keys
                .secret
                .decrypt(data.as_ref(), &keys.public)
                .map_err(|_| ServiceError::Decrypt);
            let _ = tx.send(res);
        });
        Box::pin(async move {
            rx.await.unwrap_or_else(|_| {
                Err(ServiceError::InternalError(
                    "failed to receive response from thread-pool".to_string(),
                ))
            })
        })
    }
}

#[derive(Clone)]
pub struct Decryptor(ConcurrencyLimit<RawDecryptor>);

impl Decryptor {
    pub fn new(state_engine_events: &EventSubscriber, thread_pool: Arc<ThreadPool>) -> Self {
        let limit = thread_pool.current_num_threads();
        let keys_events = state_engine_events.keys_listener();
        let service = RawDecryptor {
            keys_events,
            thread_pool,
        };
        Self(ConcurrencyLimit::new(service, limit))
    }
}

impl<T> Service<T> for Decryptor
where
    T: AsRef<[u8]> + Sync + Send + 'static,
{
    type Response = Vec<u8>;
    type Error = ServiceError;
    type Future = ResponseFuture<BoxedServiceFuture<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <ConcurrencyLimit<RawDecryptor> as Service<T>>::poll_ready(&mut self.0, cx)
    }

    fn call(&mut self, data: T) -> Self::Future {
        self.0.call(data)
    }
}
