use async_trait::async_trait;
use std::io::Error;

use crate::{
    engine::{
        model::Model,
        states::{Aggregate, Handler, State, StateCondition, StateName},
        utils::features::Features,
        Engine, ServerState,
    },
    message::Message,
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

    async fn perform(&mut self) -> Result<(), Error> {
        self.process().await?;
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(StateCondition::<Aggregate>::new(self.shared, self.private.features).into())
    }
}

impl StateCondition<Collect> {
    /// Creates a new collect state.
    pub fn new(shared: ServerState) -> Self {
        Self {
            private: Collect {
                features: Features::new(),
            },
            shared,
        }
    }
    /// Add message to feature list.
    fn add(&mut self, req: Message) -> Result<(), Error> {
        let mut local_model: Model = Default::default();
        local_model.deserialize(req.data, &self.shared.round_params.dtype);
        self.private.features.increment(&1);

        Ok(self.private.features.locals.push(local_model))
    }
}

#[async_trait]
impl Handler for StateCondition<Collect> {
    async fn handle_request(&mut self, req: Message) -> Result<(), Error> {
        self.add(req)
    }
}
