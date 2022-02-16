use crate::core::model::Model;
use rayon::prelude::*;
use rug::Rational;

#[derive(Debug, Default)]
pub struct Features {
    /// keeps msgs in cache that have been received by the clients.
    pub locals: Vec<Model>,
    /// keeps track of the number of samples each model was trained on which will result in a weighting factor.
    pub stakes: Vec<u32>,
    /// stores the overall averaged vector of all messages.
    pub global: Model,
    /// stores m_t for current iteration
    pub m_t: Model,
    /// stores v_t for current iteration.
    pub v_t: Model,
}

impl Features {
    /// Instantiates new ['Features'] object.
    pub fn new(locals: Vec<Model>, stakes: Vec<u32>) -> Self {
        Features {
            locals,
            stakes,
            global: Default::default(),
            m_t: Default::default(),
            v_t: Default::default(),
        }
    }
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
    pub fn prep_stakes(&self) -> Vec<Rational> {
        let all = self.sum_stakes();
        self.stakes
            .par_iter()
            .map(|s| Rational::from_f32(*s as f32 / all as f32).unwrap_or_else(Rational::new))
            .collect::<Vec<_>>()
    }
    /// Returns the sum of all elements in stakes.
    fn sum_stakes(&self) -> u32 {
        self.stakes.par_iter().sum()
    }
}
