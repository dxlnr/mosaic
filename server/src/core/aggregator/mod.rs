pub mod features;
pub mod traits;

use num::rational::Ratio;
use num_bigint::ToBigInt;
use rayon::prelude::*;
use std::ops::{Add, Div};

use self::traits::FedAdam;
use crate::core::model::Model;

pub enum Scheme {
    FedAvg,
    FedAdam,
}

/// Parameters that fascilitate the aggregation schema.
#[derive(Debug, Clone)]
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
pub struct Aggregator<S> {
    /// Generic strategy that sits on top of FedAvg
    pub strategy: S,
}

impl<S> Aggregator<S> {
    fn add(features: Vec<Model>) -> (Model, usize) {
        let mut res = Model::zeros(&features[0].len());

        features
            .iter()
            .map(|single| {
                res.0 = res
                    .0
                    .par_iter()
                    .zip(&single.0)
                    .map(|(w1, w2)| w1.add(w2))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
        (res, features.len())
    }

    fn avg(mut model: Model, stakes: Vec<u32>, feat_len: u32) -> Model {
        let avg_factor = Ratio::from_integer(feat_len.to_bigint().unwrap());
        model.0 = model
            .0
            .par_iter()
            .map(|w| w.div(&avg_factor))
            .collect::<Vec<_>>()
            .to_vec();
        model
    }

    pub fn aggregate(features: Vec<Model>) -> Model {
        todo!()
    }
}

impl<S> FedAdam for Aggregator<S>
where
    S: FedAdam,
{
    fn adapt(&mut self) -> Model {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::{bigint::BigInt, rational::Ratio, traits::One};

    #[test]
    fn test_add() {
        let m1 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);
        let m2 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);
        let m3 = Model(vec![
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
            Ratio::<BigInt>::one(),
        ]);
        let model_list = vec![m1, m2, m3];

        let (new_m, _) = Aggregator::<String>::add(model_list);
        assert_eq!(
            new_m,
            Model(vec![
                Ratio::from_float(3.0_f32).unwrap(),
                Ratio::from_float(3.0_f32).unwrap(),
                Ratio::from_float(3.0_f32).unwrap()
            ])
        )
    }
}
