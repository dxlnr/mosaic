use crate::core::model::Model;
use num::{bigint::BigInt, rational::Ratio, traits::Zero};
use num_bigint::ToBigInt;
use rayon::prelude::*;

use super::Aggregator;

#[derive(Debug)]
pub struct Features {
    /// keeps msgs in cache that have been received by the clients.
    pub locals: Vec<Model>,
    /// keeps track of the number of samples each model was trained on which will result in a weighting factor.
    pub stakes: Vec<u32>,
    /// stores the overall averaged vector of all messages.
    pub global: Model,
    /// aggregation object
    pub aggregator: Aggregator,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            locals: Vec::new(),
            stakes: Vec::new(),
            global: Default::default(),
            aggregator: Aggregator,
        }
    }
}

impl Features {
    /// Instantiates new ['Features'] object.
    pub fn new(locals: Vec<Model>, stakes: Vec<u32>) -> Self {
        Features {
            locals,
            stakes,
            global: Default::default(),
            aggregator: Aggregator,
        }
    }
    /// Returns number of overall local models as Ratio<BigInt>
    pub fn number_of_local_feat(&self) -> Ratio<BigInt> {
        Ratio::from_integer(self.locals.len().to_bigint().unwrap())
    }
    /// Returns a list of factors that represents the stake of each model to the global model.
    /// Computed by the number of samples it is trained on.
    pub fn prep_stakes(&self) -> Vec<Ratio<BigInt>> {
        let all = self.sum_stakes();
        self.stakes
            .par_iter()
            .map(|s| {
                Ratio::from_float(*s as f32 / all as f32).unwrap_or_else(Ratio::<BigInt>::zero)
            })
            .collect::<Vec<_>>()
    }
    /// Returns the sum of all elements in stakes.
    fn sum_stakes(&self) -> u32 {
        self.stakes.par_iter().sum()
    }
}
