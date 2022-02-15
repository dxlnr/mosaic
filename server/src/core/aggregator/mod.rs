pub mod features;
pub mod traits;

use num::{bigint::BigInt, rational::Ratio, traits::Zero};
use rayon::prelude::*;
use std::ops::{Add, Mul};

use crate::core::model::Model;

use self::{
    features::Features,
    traits::{Aggregator, FedAvg, Strategy},
};

#[derive(Debug)]
pub enum Aggregation {
    FedAvg(Aggregator<FedAvg>),
    // FedAdam(Aggregator<FedAdam>),
}

impl Aggregation {
    pub fn aggregate(&mut self) -> Model {
        match self {
            Aggregation::FedAvg(strategy) => strategy.aggregate(),
            // Scheme::FedAdam => state.run_state().await,
        }
    }
    pub fn set_feat(self, features: Features) {
        match self {
            Aggregation::FedAvg(mut strategy) => strategy.set_feat(features),
        }
    }
}

/// Parameters that fascilitate the aggregation schema.
#[derive(Debug, Clone, Copy)]
pub struct AggregationParams {
    /// Server-side learning rate. Defaults to 1e-1.
    pub eta: f64,
    /// Momentum parameter. Defaults to 0.9
    pub beta_1: f64,
    /// Second moment parameter. Defaults to 0.99.
    pub beta_2: f64,
    ///  Controls the algorithm's degree of adaptability. Defaults to 1e-9.
    pub tau: f64,
}

impl AggregationParams {
    pub fn new(eta: f64, beta_1: f64, beta_2: f64, tau: f64) -> Self {
        Self {
            eta,
            beta_1,
            beta_2,
            tau,
        }
    }
    pub fn get_beta_1(&self) -> Ratio<BigInt> {
        Ratio::from_float(self.beta_1).unwrap_or_else(Ratio::<BigInt>::zero)
    }
    pub fn get_beta_2(&self) -> Ratio<BigInt> {
        Ratio::from_float(self.beta_2).unwrap_or_else(Ratio::<BigInt>::zero)
    }
    pub fn get_eta(&self) -> Ratio<BigInt> {
        Ratio::from_float(self.eta).unwrap_or_else(Ratio::<BigInt>::zero)
    }
    pub fn get_tau(&self) -> Ratio<BigInt> {
        Ratio::from_float(self.tau).unwrap_or_else(Ratio::<BigInt>::zero)
    }
}

impl Default for AggregationParams {
    /// Setting default values for the aggregation parameters.
    fn default() -> Self {
        Self {
            eta: 1e-1,
            beta_1: 0.9,
            beta_2: 0.99,
            tau: 1e-9,
        }
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct Baseline {
    pub params: AggregationParams,
}

impl Baseline {
    // /// Creates a new [`Baseline`].
    pub fn new(params: AggregationParams) -> Self {
        Self { params }
    }
    /// Performs FedAvg and returns an aggregated model.
    pub fn avg(&mut self, features: &[Model], stakes: &[Ratio<BigInt>]) -> Model {
        let mut res = Model::zeros(&features[0].len());

        features
            .iter()
            .zip(stakes)
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

        let mut agg_object = Baseline::default();
        let new_m = agg_object.avg(&feats.locals, &feats.prep_stakes());
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
