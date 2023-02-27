use std::task::Poll;

use futures::{future, task::Context};
use tower::Service;

use crate::{
    services::messages::ServiceError,
    state_engine::events::{EventListener, EventSubscriber},
};
use mosaic_core::{
    common::RoundParameters,
    message::{Message, Payload},
};

/// A service for performing sanity checks and preparing incoming
/// requests to be handled by the state machine.
#[derive(Clone, Debug)]
pub struct TaskValidator {
    params_listener: EventListener<RoundParameters>,
}

impl TaskValidator {
    pub fn new(subscriber: &EventSubscriber) -> Self {
        Self {
            params_listener: subscriber.params_listener(),
        }
    }
}

impl Service<Message> for TaskValidator {
    type Response = Message;
    type Error = ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, message: Message) -> Self::Future {
        #[cfg(feature = "secure")]
        let (sum_signature, update_signature) = match message.payload {
            Payload::Sum(ref sum) => (sum.sum_signature, None),
            Payload::Update(ref update) => (update.sum_signature, Some(update.update_signature)),
            Payload::Sum2(ref sum2) => (sum2.sum_signature, None),
            _ => return future::ready(Err(ServiceError::UnexpectedMessage)),
        };
        #[cfg(not(feature = "secure"))]
        let _update_signature = match message.payload {
            Payload::Sum(ref _sum) => unimplemented!(),
            Payload::Update(ref update) => update.update_signature,
            Payload::Sum2(ref _sum2) => unimplemented!(),
            _ => return future::ready(Err(ServiceError::UnexpectedMessage)),
        };

        #[cfg(feature = "secure")]
        {
            let params = self.params_listener.get_latest().event;
            let seed = params.seed.as_slice();
            
            // Check whether the participant is eligible for the update task
            let has_valid_update_signature = message
                .participant_pk
                .verify_detached(&update_signature, &[seed, b"update"].concat());

            // // Check whether the participant is eligible for the sum task
            // let has_valid_sum_signature = message
            //     .participant_pk
            //     .verify_detached(&sum_signature, &[seed, b"sum"].concat());
            // let is_summer = has_valid_sum_signature && sum_signature.is_eligible(params.sum);

            // Check whether the participant is eligible for the update task
            // let has_valid_update_signature = update_signature
            //     .map(|sig| {
            //         message
            //             .participant_pk
            //             .verify_detached(&sig, &[seed, b"update"].concat())
            //     })
            //     .unwrap_or(false);

            // // Check whether the participant is eligible for the update task
            // let has_valid_update_signature = message
            //     .participant_pk
            //     .verify_detached(&update_signature, &[seed, b"update"].concat());

            // let is_updater = !is_summer
            //     && has_valid_update_signature
            //     && update_signature
            //         .map(|sig| sig.is_eligible(params.update))
            //         .unwrap_or(false);
        }
        let is_summer = false;
        let is_updater = true;

        match message.payload {
            Payload::Sum(_) | Payload::Sum2(_) => {
                if is_summer {
                    future::ready(Ok(message))
                } else {
                    future::ready(Err(ServiceError::NotSumEligible))
                }
            }
            Payload::Update(_) => {
                if is_updater {
                    future::ready(Ok(message))
                } else {
                    future::ready(Err(ServiceError::NotUpdateEligible))
                }
            }
            _ => future::ready(Err(ServiceError::UnexpectedMessage)),
        }
    }
}
