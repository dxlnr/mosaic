use async_trait::async_trait;

use crate::{
    core::{
        aggregator::features::{FeatureMap, Features},
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
    pub feature_map: FeatureMap,
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
            StateCondition::<Aggregate>::new(self.shared, self.cache, self.private.feature_map)
                .into(),
        )
    }
}

impl StateCondition<Collect> {
    /// Creates a new [`Collect`] state.
    pub fn new(shared: ServerState, mut feature_map: FeatureMap, mut cache: Cache) -> Self {
        cache.set_round_id(cache.get_round_id() + 1);

        if let Some(x) = feature_map.get_mut(&cache.get_round_id()) {
                x.set_global_mt_vt(cache.global_model.clone(),cache.m_t.clone(),cache.v_t.clone());
        } else {
            feature_map.insert_into(cache.get_round_id(), Features::new_cached(
                cache.global_model.clone(),
                cache.m_t.clone(),
                cache.v_t.clone()));
        }

        Self {
            private: Collect { feature_map },
            shared,
            cache,
        }
    }
    /// Add message to feature list.
    fn add(&mut self, req: Message) -> Result<(), ServiceError> {
        let mut local_model: Model = Default::default();
        local_model.deserialize(req.data, &self.shared.round_params.dtype);

        if let Some(features) = self.private.feature_map.get_mut(&req.model_version) {
            features.locals.push(local_model);
            features.stakes.push(req.stake);
        } else {
            self.private.feature_map.insert_into(req.model_version, Features::new(vec![local_model], vec![req.stake]));
        }

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
        mut feature_map: FeatureMap,
        req: Message,
        local_model: Model,
    ) -> FeatureMap {
        if let Some(features) = feature_map.get_mut(&req.model_version) {
            features.locals.push(local_model);
            features.stakes.push(req.stake);
        } else {
            feature_map.insert_into(req.model_version, Features::new(vec![local_model], vec![req.stake]));
        }
        feature_map
    }

    #[test]
    fn test_collect_add() {
        let mut test_vec_map = FeatureMap::default();
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

        test_vec_map = modified_add(test_vec_map, msg_one.clone(), local_model.clone());
        assert_eq!(
            test_vec_map.fmap.get(&1).unwrap().locals[0],
            Model(vec![Float::with_val(32, 0.25), Float::with_val(32, 0.25)])
        );

        // println!("{:?}", &test_vec_map);

        test_vec_map = modified_add(test_vec_map, msg_one.clone(), local_model.clone());
        assert_eq!(
            test_vec_map.fmap.get(&1).unwrap().locals[1],
            Model(vec![Float::with_val(32, 0.25), Float::with_val(32, 0.25)])
        );

        // println!("{:?}", &test_vec_map);
    }
}
