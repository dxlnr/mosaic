//! Feature module

use crate::core::model::Model;
use rayon::prelude::*;
use rug::Float;

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
    /// Returns a list of factors that represents the stake of each model to the global model.
    /// Computed by the number of samples it is trained on.
    pub fn prep_stakes(&self) -> Vec<Float> {
        let all = self.sum_stakes();
        self.stakes
            .par_iter()
            .map(|s| Float::with_val(53, *s / all))
            .collect::<Vec<_>>()
    }
    /// Returns the sum of all elements in stakes.
    fn sum_stakes(&self) -> u32 {
        self.stakes.par_iter().sum()
    }
}
