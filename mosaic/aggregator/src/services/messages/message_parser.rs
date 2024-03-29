use std::{convert::TryInto, sync::Arc, task::Poll};

use futures::{future, task::Context};
use rayon::ThreadPool;
use tokio::sync::oneshot;
use tower::{layer::Layer, limit::concurrency::ConcurrencyLimit, Service, ServiceBuilder};
use tracing::{debug, trace, warn};

use crate::{
    services::messages::{BoxedServiceFuture, ServiceError},
    state_engine::{
        events::{EventListener, EventSubscriber},
        states::StateName,
    },
};
use mosaic_core::{
    crypto::{EncryptKeyPair, PublicEncryptKey},
    message::{FromBytes, Message, MessageBuffer, Tag},
};

/// A type that hold a un-parsed message
struct RawMessage<T> {
    /// The buffer that contains the message to parse
    buffer: Arc<MessageBuffer<T>>,
}

impl<T> Clone for RawMessage<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

impl<T> From<MessageBuffer<T>> for RawMessage<T> {
    fn from(buffer: MessageBuffer<T>) -> Self {
        RawMessage {
            buffer: Arc::new(buffer),
        }
    }
}

/// A service that wraps a buffer `T` representing a message into a
/// [`RawMessage<T>`]
#[derive(Debug, Clone)]
struct BufferWrapper<S>(S);

impl<S, T> Service<T> for BufferWrapper<S>
where
    T: AsRef<[u8]> + Send + 'static,
    S: Service<RawMessage<T>, Response = Message, Error = ServiceError>,
    S::Future: Sync + Send + 'static,
{
    type Response = Message;
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: T) -> Self::Future {
        debug!("Creating a raw message request.");
        match MessageBuffer::new(req) {
            Ok(buffer) => {
                let fut = self.0.call(RawMessage::from(buffer));
                Box::pin(async move {
                    trace!("Calling inner service.");
                    fut.await
                })
            }
            Err(e) => Box::pin(future::ready(Err(ServiceError::Parsing(e)))),
        }
    }
}

struct BufferWrapperLayer;

impl<S> Layer<S> for BufferWrapperLayer {
    type Service = BufferWrapper<S>;

    fn layer(&self, service: S) -> BufferWrapper<S> {
        BufferWrapper(service)
    }
}

/// A service that discards messages that are not expected in the current phase
#[derive(Debug, Clone)]
struct PhaseFilter<S> {
    /// A listener to retrieve the current phase
    phase: EventListener<StateName>,
    /// Next service to be called
    next_svc: S,
}

impl<T, S> Service<RawMessage<T>> for PhaseFilter<S>
where
    T: AsRef<[u8]> + Send + 'static,
    S: Service<RawMessage<T>, Response = Message, Error = ServiceError>,
    S::Future: Sync + Send + 'static,
{
    type Response = Message;
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next_svc.poll_ready(cx)
    }

    fn call(&mut self, req: RawMessage<T>) -> Self::Future {
        debug!("Retrieving the current state.");
        let phase = self.phase.get_latest().event;
        match req.buffer.tag().try_into() {
            // Ok(tag) => match (phase, tag) {
            //     (StateName::Sum, Tag::Sum)
            //     | (StateName::Update, Tag::Update)
            //     | (StateName::Sum2, Tag::Sum2) => {
            //         let fut = self.next_svc.call(req);
            //         Box::pin(async move { fut.await })
            //     }
            //     _ => Box::pin(future::ready(Err(ServiceError::UnexpectedMessage))),
            // },
            Ok(tag) => match (phase, tag) {
                (StateName::Update, Tag::Update)=> {
                    let fut = self.next_svc.call(req);
                    Box::pin(async move { fut.await })
                }
                _ => Box::pin(future::ready(Err(ServiceError::UnexpectedMessage))),
            },
            Err(e) => Box::pin(future::ready(Err(ServiceError::Parsing(e)))),
        }
    }
}

struct PhaseFilterLayer {
    phase: EventListener<StateName>,
}

impl<S> Layer<S> for PhaseFilterLayer {
    type Service = PhaseFilter<S>;

    fn layer(&self, service: S) -> PhaseFilter<S> {
        PhaseFilter {
            phase: self.phase.clone(),
            next_svc: service,
        }
    }
}

/// A service for verifying the signature of PET messages
///
/// Since this is a CPU-intensive task for large messages, this
/// service offloads the processing to a `rayon` thread-pool to avoid
/// overloading the tokio thread-pool with blocking tasks.
#[derive(Debug, Clone)]
struct SignatureVerifier<S> {
    /// Thread-pool the CPU-intensive tasks are offloaded to.
    thread_pool: Arc<ThreadPool>,
    /// The service to be called after the [`SignatureVerifier`]
    next_svc: S,
}

impl<T, S> Service<RawMessage<T>> for SignatureVerifier<S>
where
    T: AsRef<[u8]> + Sync + Send + 'static,
    S: Service<RawMessage<T>, Response = Message, Error = ServiceError>
        + Clone
        + Sync
        + Send
        + 'static,
    S::Future: Sync + Send + 'static,
{
    type Response = Message;
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next_svc.poll_ready(cx)
    }

    fn call(&mut self, req: RawMessage<T>) -> Self::Future {
        let (tx, rx) = oneshot::channel::<Result<(), ServiceError>>();

        let req_clone = req.clone();
        trace!("Spawning signature verification task on thread-pool.");
        self.thread_pool.spawn(move || {
            let res = match req.buffer.as_ref().as_ref().check_signature() {
                Ok(()) => {
                    debug!("Found a valid message signature.");
                    Ok(())
                }
                Err(e) => {
                    warn!("Invalid message signature: {:?}.", e);
                    Err(ServiceError::InvalidMessageSignature)
                }
            };
            let _ = tx.send(res);
        });

        let mut next_svc = self.next_svc.clone();
        let fut = async move {
            rx.await.map_err(|_| {
                ServiceError::InternalError(
                    "failed to receive response from thread-pool".to_string(),
                )
            })??;
            next_svc.call(req_clone).await
        };
        Box::pin(fut)
    }
}

struct SignatureVerifierLayer {
    thread_pool: Arc<ThreadPool>,
}

impl<S> Layer<S> for SignatureVerifierLayer {
    type Service = ConcurrencyLimit<SignatureVerifier<S>>;

    fn layer(&self, service: S) -> Self::Service {
        let limit = self.thread_pool.current_num_threads();
        // FIXME: we actually want to limit the concurrency of just
        // the SignatureVerifier middleware. Right now we're limiting
        // the whole stack of services.
        ConcurrencyLimit::new(
            SignatureVerifier {
                thread_pool: self.thread_pool.clone(),
                next_svc: service,
            },
            limit,
        )
    }
}

/// A service that verifies the coordinator public key embedded in PET
/// messsages
#[derive(Debug, Clone)]
struct CoordinatorPublicKeyValidator<S> {
    /// A listener to retrieve the latest coordinator keys
    keys: EventListener<EncryptKeyPair>,
    /// Next service to be called
    next_svc: S,
}

impl<T, S> Service<RawMessage<T>> for CoordinatorPublicKeyValidator<S>
where
    T: AsRef<[u8]> + Send + 'static,
    S: Service<RawMessage<T>, Response = Message, Error = ServiceError>,
    S::Future: Sync + Send + 'static,
{
    type Response = Message;
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.next_svc.poll_ready(cx)
    }

    fn call(&mut self, req: RawMessage<T>) -> Self::Future {
        debug!("Retrieving the current keys.");
        let coord_pk = self.keys.get_latest().event.public;
        match PublicEncryptKey::from_byte_slice(&req.buffer.as_ref().as_ref().coordinator_pk()) {
            Ok(pk) => {
                if pk != coord_pk {
                    warn!("Found an Invalid aggregator public key.");
                    Box::pin(future::ready(Err(
                        ServiceError::InvalidCoordinatorPublicKey,
                    )))
                } else {
                    debug!("Found a valid aggregator public key.");
                    let fut = self.next_svc.call(req);
                    Box::pin(async move { fut.await })
                }
            }
            Err(_) => Box::pin(future::ready(Err(
                ServiceError::InvalidCoordinatorPublicKey,
            ))),
        }
    }
}

struct CoordinatorPublicKeyValidatorLayer {
    keys: EventListener<EncryptKeyPair>,
}

impl<S> Layer<S> for CoordinatorPublicKeyValidatorLayer {
    type Service = CoordinatorPublicKeyValidator<S>;

    fn layer(&self, service: S) -> CoordinatorPublicKeyValidator<S> {
        CoordinatorPublicKeyValidator {
            keys: self.keys.clone(),
            next_svc: service,
        }
    }
}

#[derive(Debug, Clone)]
struct Parser;

impl<T> Service<RawMessage<T>> for Parser
where
    T: AsRef<[u8]> + Send + 'static,
{
    type Response = Message;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RawMessage<T>) -> Self::Future {
        let bytes = req.buffer.inner();
        future::ready(Message::from_byte_slice(&bytes).map_err(ServiceError::Parsing))
    }
}

// type InnerService = BufferWrapper<
//     PhaseFilter<ConcurrencyLimit<SignatureVerifier<CoordinatorPublicKeyValidator<Parser>>>>,
// >;
type InnerService = BufferWrapper<
    ConcurrencyLimit<SignatureVerifier<CoordinatorPublicKeyValidator<Parser>>>,
>;

#[derive(Debug, Clone)]
pub struct MessageParser(InnerService);

impl<T> Service<T> for MessageParser
where
    T: AsRef<[u8]> + Sync + Send + 'static,
{
    type Response = Message;
    type Error = ServiceError;
    type Future = BoxedServiceFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        <InnerService as Service<T>>::poll_ready(&mut self.0, cx)
    }

    fn call(&mut self, req: T) -> Self::Future {
        let fut = self.0.call(req);
        Box::pin(async move { fut.await })
    }
}

impl MessageParser {
    #[cfg(feature = "secure")]
    pub fn new(events: &EventSubscriber, thread_pool: Arc<ThreadPool>) -> Self {
        let inner = ServiceBuilder::new()
            .layer(BufferWrapperLayer)
            .layer(PhaseFilterLayer {
                phase: events.state_listener(),
            })
            .layer(SignatureVerifierLayer { thread_pool })
            .layer(CoordinatorPublicKeyValidatorLayer {
                keys: events.keys_listener(),
            })
            .service(Parser);
        Self(inner)
    }
    #[cfg(not(feature = "secure"))]
    pub fn new(events: &EventSubscriber, thread_pool: Arc<ThreadPool>) -> Self {
        let inner = ServiceBuilder::new()
            .layer(BufferWrapperLayer)
            // .layer(PhaseFilterLayer {
            //     phase: events.state_listener(),
            // })
            .layer(SignatureVerifierLayer { thread_pool })
            .layer(CoordinatorPublicKeyValidatorLayer {
                keys: events.keys_listener(),
            })
            .service(Parser);
        Self(inner)
    }
}
