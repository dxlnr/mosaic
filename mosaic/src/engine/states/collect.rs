use async_trait::async_trait;

use crate::{
    core::{aggregator::features::{Features, FeatureDeque}, model::Model},
    engine::{
        states::{error::StateError, Aggregate, Handler, State, StateCondition, StateName},
        Cache, Engine, ServerState,
    },
    proxy::message::Message,
    rest::stats::Single,
    service::error::ServiceError,
};

#[derive(Debug)]
/// [`Collect`] object representing the collect state.
pub struct Collect {
    /// Caches all the incoming messages and their respective data.
    pub features: Features,
    // pub feature_deque: FeatureDeque,
}

#[async_trait]
impl State for StateCondition<Collect>
where
    Self: Handler,
{
    const NAME: StateName = StateName::Collect;

    async fn perform(&mut self) -> Result<(), StateError> {
        self.process().await?;

        self.shared
            .publisher
            .broadcast_stats(Some(self.cache.stats.clone()));
        Ok(())
    }

    async fn next(self) -> Option<Engine> {
        Some(
            StateCondition::<Aggregate>::new(self.shared, self.cache, self.private.features).into(),
        )
    }
}

impl StateCondition<Collect> {
    /// Creates a new [`Collect`] state.
    // pub fn new(shared: ServerState, mut feature_deque: FeatureDeque, mut cache: Cache) -> Self {
    pub fn new(shared: ServerState, mut cache: Cache) -> Self {
        cache.set_round_id(cache.get_round_id() + 1);
        Self {
            private: Collect {
                features: Features::new_cached(
                    cache.global_model.clone(),
                    cache.m_t.clone(),
                    cache.v_t.clone(),
                ),
            },
            shared,
            cache,
        }
    }
    /// Add message to feature list.
    fn add(&mut self, req: Message) -> Result<(), ServiceError> {
        let mut local_model: Model = Default::default();
        local_model.deserialize(req.data, &self.shared.round_params.dtype);

        self.private.features.locals.push(local_model);
        self.private.features.stakes.push(req.stake);

        self.cache.stats.msgs.push(Single::new(
            req.key,
            self.cache.round_id,
            req.loss,
            req.stake,
        ));
        Ok(())
    }
}

#[async_trait]
impl Handler for StateCondition<Collect> {
    async fn handle_request(&mut self, req: Message) -> Result<(), ServiceError> {
        self.add(req)
    }
}
