//! Feature module which stores all important information enabling the aggregation regarding each training round.
//!
//! Some attributes have to be cached for the consecutive round as well like the global, m_t & v_t model.
use crate::core::model::Model;
use rayon::prelude::*;
use rug::Float;
use std::collections::VecDeque;

#[derive(Debug, Default, Clone)]
// TODO: Probably hashmap might be beneficial.
pub struct FeatureDeque {
    pub queue: VecDeque<Features>,
}

impl FeatureDeque {
    /// Provides a reference to the element at the given index.
    ///
    /// Element at index 0 is the front of the queue.
    pub fn get(&self, index: usize) -> Option<&Features> {
        self.queue.get(index)
    }
    /// Provides a mutable reference to the element at the given index.
    ///
    /// Element at index 0 is the front of the queue.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Features> {
        self.queue.get_mut(index)
    }
    /// Removes the first element and returns it, or `None` if qeque is empty.
    pub fn pop_front(&mut self) -> Option<Features> {
        self.queue.pop_front()
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
