use async_trait::async_trait;

use crate::{
    core::{aggregator::features::Features, model::Model},
    engine::{
        states::{error::StateError, Aggregate, Handler, State, StateCondition, StateName},
        Engine, ServerState,
    },
    proxy::message::Message,
    service::error::ServiceError,
};

/// The collect state.
#[derive(Debug)]
pub struct Collect {
    /// Caches all the incoming messages and their respective data.
    pub features: Features,
}

#[async_trait]
impl State for StateCondition<Collect>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), StateError> {
        self.process().await?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Aggregate>::new(self.shared, self.private.features).into())
    }
}

impl StateCondition<Collect> {
    /// Creates a new collect state.
    pub fn new(mut shared: ServerState) -> Self {
        shared.set_round_id(shared.round_id() + 1);
        Self {
            private: Collect {
                features: Features::default(),
            },
            shared,
        }
    }
    /// Add message to feature list.
    fn add(&mut self, req: Message) -> Result<(), ServiceError> {
        let mut local_model: Model = Default::default();
        local_model.deserialize(req.data, &self.shared.round_params.dtype);

        self.private.features.locals.push(local_model);
        self.private.features.stakes.push(req.stake);
        Ok(())
    }
}

#[async_trait]
impl Handler for StateCondition<Collect> {
    async fn handle_request(&mut self, req: Message) -> Result<(), ServiceError> {
        self.add(req)
    }
}
