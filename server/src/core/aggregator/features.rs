use crate::core::model::Model;
use num::{bigint::BigInt, rational::Ratio};
use num_bigint::ToBigInt;
use rayon::prelude::*;
use std::ops::{Add, Div};

use super::{Aggregator};

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
    pub fn new() -> Self {
        Features {
            locals: Vec::new(),
            stakes: Vec::new(),
            global: Default::default(),
            aggregator: Aggregator,
        }
    }
    // /// Increment the factor which holds the number of received messages from previous.
    // pub fn increment(&mut self, weight: &u32) {
    //     self.factor += weight;
    // }
    /// Returns number of overall local models as Ratio<BigInt>
    pub fn number_of_local_feat(length: u32) -> Ratio<BigInt> {
        Ratio::from_integer(length.to_bigint().unwrap())
    }
    /// Returns a list of factors that represents the stake of each model to the global model.
    /// Computed by the number of samples it is trained on.
    pub fn prep_stakes() -> Vec<Ratio<BigInt>> {
        todo!()
    }
    /// Elementwise adding of (all) single models to one global model for particular training round.
    pub fn add(&mut self) {
        self.global.0 = self.locals[0].0.clone();

        self.locals
            .iter()
            .skip(1)
            .map(|single| {
                self.global.0 = self
                    .global
                    .0
                    .par_iter()
                    .zip(&single.0)
                    .map(|(w1, w2)| w1.add(w2))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
    }
    // /// Averaging the summed global part of ['Features'].
    // pub fn avg(&mut self) {
    //     let avg_factor = Ratio::from_float(self.factor as f32)
    //         .unwrap_or_else(|| Ratio::from_float(1.0).unwrap());
    //     self.global.0 = self
    //         .global
    //         .0
    //         .par_iter()
    //         .map(|w| w.div(&avg_factor))
    //         .collect::<Vec<_>>()
    //         .to_vec();
    // }
}
