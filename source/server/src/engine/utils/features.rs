use crate::engine::model::Model;
use num::rational::Ratio;
use std::ops::{Add, Div};

#[derive(Debug, Default)]
pub struct Features {
    // /// keeps msgs in cache that have been received by the clients.
    pub locals: Vec<Model>,
    /// keeps track of the number of model received by the clients by a weight factor.
    pub factor: u32,
    // /// Will store the overall averaged vector of all messages.
    pub global: Model,
}

impl Features {
    /// Instantiates new ['Features'] object.
    pub fn new() -> Self {
        Features {
            locals: Vec::new(),
            factor: 0,
            global: Default::default(),
        }
    }
    /// Increment the factor which holds the number of received messages from previous.
    pub fn increment(&mut self, weight: &u32) {
        self.factor += weight;
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
                    .iter()
                    .zip(&single.0)
                    .map(|(l1, l2)| {
                        l2.iter()
                            .zip(l1)
                            .map(|(w1, w2)| w1.add(w2))
                            .collect::<Vec<_>>()
                            .to_vec()
                    })
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
    }
    /// Averaging the summed global part of ['Features'].
    pub fn avg(&mut self) {
        let avg_factor =
            Ratio::from_float(self.factor as f32).unwrap_or_else(||Ratio::from_float(1.0).unwrap());
        self.global.0 = self
            .global
            .0
            .iter()
            .map(|l| {
                l.iter()
                    .map(|w| w.div(&avg_factor))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
}
