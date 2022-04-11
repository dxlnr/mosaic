use async_trait::async_trait;
// use tracing::warn;

use crate::{
    core::{
        aggregator::features::{FeatureDeque, Features},
        model::Model,
    },
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
    // pub features: Features,
    pub feature_deque: FeatureDeque,
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
            // StateCondition::<Aggregate>::new(self.shared, self.cache, self.private.features).into(),
            StateCondition::<Aggregate>::new(self.shared, self.cache, self.private.feature_deque)
                .into(),
        )
    }
}

impl StateCondition<Collect> {
    /// Creates a new [`Collect`] state.
    pub fn new(shared: ServerState, mut feature_deque: FeatureDeque, mut cache: Cache) -> Self {
        cache.set_round_id(cache.get_round_id() + 1);

        match feature_deque.get_mut(0) {
            Some(features) => features.set_global_mt_vt(
                cache.global_model.clone(),
                cache.m_t.clone(),
                cache.v_t.clone(),
            ),
            None => feature_deque.queue.push_back(Features::new_cached(
                cache.global_model.clone(),
                cache.m_t.clone(),
                cache.v_t.clone(),
            )),
        }
        // features: Features::new_cached(
        //     cache.global_model.clone(),
        //     cache.m_t.clone(),
        //     cache.v_t.clone(),

        Self {
            private: Collect { feature_deque },
            shared,
            cache,
        }
    }
    /// Add message to feature list.
    fn add(&mut self, req: Message) -> Result<(), ServiceError> {
        let mut local_model: Model = Default::default();
        local_model.deserialize(req.data, &self.shared.round_params.dtype);

        match self
            .private
            .feature_deque
            .get_mut(req.model_version.try_into().unwrap())
        {
            Some(features) => {
                features.locals.push(local_model);
                features.stakes.push(req.stake);
            }
            None => {
                self.private
                    .feature_deque
                    .queue
                    .push_back(Features::new(vec![local_model], vec![req.stake]));
            }
        }

        // self.private.features.locals.push(local_model);
        // self.private.features.stakes.push(req.stake);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::DataType;
    use rug::Float;

    fn modified_add(
        mut feature_deque: FeatureDeque,
        req: Message,
        local_model: Model,
    ) -> FeatureDeque {
        match feature_deque.get_mut(req.model_version.try_into().unwrap()) {
            Some(features) => {
                features.locals.push(local_model);
                features.stakes.push(req.stake);
            }
            None => {
                feature_deque
                    .queue
                    .push_back(Features::new(vec![local_model], vec![req.stake]));
            }
        }
        feature_deque
    }

    #[test]
    fn test_collect_add() {
        let mut test_vec_deque = FeatureDeque::default();
        let msg_one = Message::new(
            1,
            1,
            vec![62, 128, 0, 0, 62, 128, 0, 0],
            DataType::F32,
            1,
            0.7,
        );

        let mut local_model: Model = Default::default();
        local_model.deserialize(msg_one.data.clone(), &msg_one.dtype);

        test_vec_deque = modified_add(test_vec_deque, msg_one.clone(), local_model.clone());
        assert_eq!(
            test_vec_deque.queue[0].locals[0],
            Model(vec![Float::with_val(32, 0.25), Float::with_val(32, 0.25)])
        );

        test_vec_deque = modified_add(test_vec_deque, msg_one.clone(), local_model.clone());
        assert_eq!(
            test_vec_deque.queue[1].locals[0],
            Model(vec![Float::with_val(32, 0.25), Float::with_val(32, 0.25)])
        );
    }
}
