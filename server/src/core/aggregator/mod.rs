pub mod features;
pub mod traits;

use num::{bigint::BigInt, rational::Ratio};
use rayon::prelude::*;
use std::ops::{Add, Mul};

use self::traits::{FedAdam, FedAvg};
use crate::core::model::Model;

pub enum Scheme {
    FedAvg,
    FedAdam,
}

/// Parameters that fascilitate the aggregation schema.
#[derive(Debug, Clone, Copy)]
pub struct AggregationParams {
    // pub delta_t: f64,
    // pub m_t: f64,
    // pub v_t: f64,
    /// Server-side learning rate. Defaults to 1e-1.
    pub eta: f64,
    /// Momentum parameter. Defaults to 0.9
    pub beta_1: f64,
    /// Second moment parameter. Defaults to 0.99.
    pub beta_2: f64,
    ///  Controls the algorithm's degree of adaptability. Defaults to 1e-9.
    pub tau_t: f64,
}

impl AggregationParams {
    pub fn new(eta: f64, beta_1: f64, beta_2: f64, tau_t: f64) -> Self {
        Self {
            eta,
            beta_1,
            beta_2,
            tau_t,
        }
    }
}

impl Default for AggregationParams {
    fn default() -> Self {
        Self {
            eta: 1e-1,
            beta_1: 0.9,
            beta_2: 0.99,
            tau_t: 1e-9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Aggregation<A> {
    /// Generic strategy that sits on top of FedAvg
    pub aggregator: A,
    // /// Aggregation params needed for performing certain algorithms.
    // pub params: AggregationParams,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct Aggregator {
    pub params: AggregationParams,
}

impl Aggregator {
    // /// Creates a new [`Aggregator`].
    pub fn new(params: AggregationParams) -> Self {
        Self { params }
    }
    /// Performs FedAvg and returns an aggregated model.
    pub fn avg(self, features: Vec<Model>, stakes: Vec<Ratio<BigInt>>) -> Model {
        let mut res = Model::zeros(&features[0].len());

        features
            .iter()
            .zip(&stakes)
            .map(|(single, s)| {
                res.0 = res
                    .0
                    .par_iter()
                    .zip(&single.0)
                    .map(|(w1, w2)| w1.add(w2.mul(s)))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
        res
    }
    // pub fn aggregate(_features: Vec<Model>) -> Model {
    //     todo!()
    // }
}

impl FedAvg for Aggregator {
    fn aggregate(&mut self, features: Vec<Model>, stakes: Vec<Ratio<BigInt>>) -> Model {
        self.avg(features, stakes)
    }
}

impl FedAdam for Aggregator {
    fn adapt(&mut self) -> Model {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use self::features::Features;
    use super::*;
    use num::{bigint::BigInt, rational::Ratio, traits::One};

    #[test]
    fn test_add() {
        let m1 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);
        let m2 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);
        let m3 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);
        let m4 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);

        let model_list = vec![m1, m2, m3, m4];
        let stakes = vec![1, 1, 1, 1];

        let feats = Features::new(model_list, stakes);

        let agg_object = Aggregator::default();
        let new_m = agg_object.avg(feats.locals.clone(), feats.prep_stakes());
        assert_eq!(
            new_m,
            Model(vec![
                // Ratio::from_float(3.0_f32).unwrap(),
                // Ratio::from_float(3.0_f32).unwrap(),
                // Ratio::from_float(3.0_f32).unwrap()
                Ratio::<BigInt>::one(),
                Ratio::<BigInt>::one(),
                Ratio::<BigInt>::one(),
                Ratio::<BigInt>::one(),
            ])
        )
    }
}
