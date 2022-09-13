//! Aggregation module.
//!
pub mod features;
pub mod fedopt;
pub mod traits;

use rayon::prelude::*;
use rug::Float;
use std::ops::{Add, Mul};
use tracing::error;

use crate::{core::model::Model, engine::states::error::StateError};

use self::traits::{Aggregator, FedAdaGrad, FedAdam, FedAvg, FedYogi, Strategy};

/// [`Aggregation`] strategy which defines the way the aggregation is performed.
///
/// Valid Options are `FedAvg`, `FedAdaGrad`, `FedAdam` `FedYogi`.
#[derive(Debug)]
pub enum Aggregation {
    FedAvg(Aggregator<FedAvg>),
    FedAdaGrad(Aggregator<FedAdaGrad>),
    FedAdam(Aggregator<FedAdam>),
    FedYogi(Aggregator<FedYogi>),
}

impl Aggregation {
    pub fn aggregate(&mut self) -> Result<(Model, Model, Model), StateError> {
        match self {
            Aggregation::FedAvg(strategy) => strategy.aggregate(),
            Aggregation::FedAdaGrad(strategy) => strategy.aggregate(),
            Aggregation::FedAdam(strategy) => strategy.aggregate(),
            Aggregation::FedYogi(strategy) => strategy.aggregate(),
        }
    }
}

/// Parameters necessary for performing an aggregation schema.
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
    /// Creates new [`AggregationParams`] which allows altering the default parameters
    /// eta, beta_1, beta_2 and tau.
    pub fn new(eta: f64, beta_1: f64, beta_2: f64, tau: f64) -> Self {
        Self {
            eta,
            beta_1,
            beta_2,
            tau,
        }
    }
    /// Returns the beta_1 parameter as [`Float`]
    pub fn get_beta_1(&self) -> Float {
        Float::with_val(53, self.beta_1)
    }
    /// Returns the beta_2 parameter as [`Float`]
    pub fn get_beta_2(&self) -> Float {
        Float::with_val(53, self.beta_2)
    }
    /// Returns the eta parameter as [`Float`]
    pub fn get_eta(&self) -> Float {
        Float::with_val(53, self.eta)
    }
    /// Returns the tau parameter as [`Float`]
    pub fn get_tau(&self) -> Float {
        Float::with_val(53, self.tau)
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
    /// Basic hyperparameters concerning the aggregation process. 
    pub params: AggregationParams,
}

impl Baseline {
    /// Creates a new [`Baseline`] with specified [`AggregationParams`].
    pub fn new(params: AggregationParams) -> Self {
        Self { params }
    }
    /// Performs FedAvg and returns an aggregated model.
    pub fn avg(&mut self, locals: &[Model], stakes: &[Float]) -> Result<Model, StateError> {
        if locals.is_empty() {
            error!("No local models available for aggregating. Please check whether the model version fits the current training round.");
            return Err(StateError::FeatureError(
                "No aggregation for current training round. Transition to Collect state again.",
            ));
        }
        let mut res = Model::zeros(&locals[0].len());

        locals
            .iter()
            .zip(stakes)
            .map(|(single, s)| {
                res.0 = res
                    .0
                    .par_iter()
                    .zip(&single.0)
                    .map(|(w1, w2)| w1.add(w2.clone().mul(s)))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use self::features::Features;
    use super::*;
    use rug::Float;

    #[test]
    fn test_add() {
        let m1 = Model(vec![
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
        ]);
        let m2 = Model(vec![
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
        ]);
        let m3 = Model(vec![
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
        ]);
        let m4 = Model(vec![
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
            Float::with_val(53, 1),
        ]);

        let model_list = vec![m1, m2, m3, m4];
        let stakes = vec![1, 1, 1, 1];

        let feats = Features::new(model_list, stakes);

        let mut agg_object = Baseline::default();
        let new_m = agg_object.avg(&feats.locals, &feats.prep_stakes()).unwrap();
        assert_eq!(
            new_m,
            Model(vec![
                Float::with_val(53, 1),
                Float::with_val(53, 1),
                Float::with_val(53, 1),
                Float::with_val(53, 1),
            ])
        )
    }
}