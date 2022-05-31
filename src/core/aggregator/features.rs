//! Feature module which stores all important information enabling the aggregation regarding each training round.
//!
//! Some attributes have to be cached for the consecutive round as well like the global, m_t & v_t model.
use crate::core::model::Model;
use rayon::prelude::*;
use rug::Float;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
/// [`FeatureMap`] that stores the features according to a certain training round
/// which is defined by the key of the Hashmap.
///
pub struct FeatureMap {
    pub fmap: HashMap<u32, Features>,
}

impl FeatureMap {
    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, index: &u32) -> Option<&mut Features> {
        self.fmap.get_mut(index)
    }
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    pub fn insert_into(&mut self, index: u32, values: Features) -> Option<Features> {
        self.fmap.insert(index, values)
    }
    /// Removes a key from the map and returning the value of the key.
    pub fn remove(&mut self, index: &u32) -> Option<Features> {
        self.fmap.remove(index)
    }
}
#[derive(Debug, Default, Clone)]
pub struct Features {
    /// keeps msgs in cache that have been received by the clients.
    pub locals: Vec<Model>,
    /// keeps track of the number of samples each model was trained on which will result in a weighting factor.
    pub stakes: Vec<u32>,
    /// stores the overall aggregated [`Model`] of all messages containing local models.
    pub global: Model,
    /// stores m_t for current iteration
    pub m_t: Model,
    /// stores v_t for current iteration.
    pub v_t: Model,
}

impl Features {
    /// Instantiates new [`Features`] object.
    ///
    /// Parameters locals and stakes can be set freely, while global, m_t & v_t are set as default.
    pub fn new(locals: Vec<Model>, stakes: Vec<u32>) -> Self {
        Features {
            locals,
            stakes,
            global: Default::default(),
            m_t: Default::default(),
            v_t: Default::default(),
        }
    }
    /// Instantiates new cached [`Features`] object.
    ///
    /// While the parameters locals and stakes are set as default vectors, global, m_t & v_t are input variables.
    pub fn new_cached(global: Model, m_t: Model, v_t: Model) -> Self {
        Self {
            locals: Vec::new(),
            stakes: Vec::new(),
            global,
            m_t,
            v_t,
        }
    }
    /// Alters the three parameters global, m_t & v_t for later use.
    pub fn set_global_mt_vt(&mut self, global: Model, m_t: Model, v_t: Model) {
        self.global = global;
        self.m_t = m_t;
        self.v_t = v_t;
    }

    /// Returns a list of factors that represents the stake of each model to the global model.
    /// Computed by the number of samples it is trained on.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rug::Float;
    /// let stakes = vec![8, 2, 2, 4];
    /// let feats = Features::new([Model::default(); 4], stakes);
    /// assert_eq!(
    ///     feats.prep_stakes(),
    ///     vec![
    ///         Float::with_val(53, 0.5),
    ///         Float::with_val(53, 0.125),
    ///         Float::with_val(53, 0.125),
    ///         Float::with_val(53, 0.25)]);
    /// ```
    pub fn prep_stakes(&self) -> Vec<Float> {
        let all = self.sum_stakes();
        self.stakes
            .par_iter()
            .map(|s| Float::with_val(53, *s as f32 / all as f32))
            .collect::<Vec<_>>()
    }
    /// Returns the sum of all elements in stakes.
    fn sum_stakes(&self) -> u32 {
        self.stakes.par_iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::aggregator::features::Features;
    use rug::Float;

    #[test]
    fn test_prep_stakes() {
        let stakes = vec![8, 2, 2, 4];
        let _placeholder = vec![Model::default()];
        let feats = Features::new(_placeholder, stakes);

        assert_eq!(
            feats.prep_stakes(),
            vec![
                Float::with_val(53, 0.5),
                Float::with_val(53, 0.125),
                Float::with_val(53, 0.125),
                Float::with_val(53, 0.25),
            ]
        );
    }
    #[test]
    fn test_set_global_mt_vt() {
        let mut feats = Features::new(vec![Model::default(); 4], vec![8, 2, 2, 4]);
        feats.set_global_mt_vt(
            Model(vec![Float::with_val(53, 2)]),
            Model(vec![Float::with_val(53, 4)]),
            Model(vec![Float::with_val(53, 3)]),
        );
        assert_eq!(feats.global, Model(vec![Float::with_val(53, 2)]));
    }
}
