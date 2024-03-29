use std::ops::{Add, Mul};
use thiserror::Error;
use tracing::error;

use num::{bigint::BigInt, rational::Ratio};

use mosaic_core::model::Model;

#[derive(Debug, Error)]
/// Errors related to the aggregation of masks and models.
pub enum AggregationError {
    #[error("the object to aggregate is invalid")]
    InvalidObject,
    #[error("No aggregation for current training round.")]
    NoModels,
    #[error("too many models were aggregated for the current configuration")]
    TooManyModels,
    #[error("too many scalars were aggregated for the current configuration")]
    TooManyScalars,
    #[error("the model to aggregate is incompatible with the current aggregated scalar")]
    ModelMismatch,
    #[error("the scalar to aggregate is incompatible with the current aggregated scalar")]
    ScalarMismatch,
}

#[derive(Debug, Default, Clone)]
pub struct Aggregation {
    // pub global_model: Model,
}

#[allow(clippy::len_without_is_empty)]
impl Aggregation {
    pub fn aggregate(&mut self, local_models: &Vec<Model>) -> Result<Model, AggregationError> {
        if local_models.is_empty() {
            error!("No local models available for aggregating.");
            return Err(AggregationError::NoModels);
        }

        let mut global_model = Model::zeros(&local_models[0].len());
        let stakes = Ratio::<BigInt>::new(
            BigInt::from(1_i64),
            BigInt::from(local_models.len() as i64),
        );

        local_models
            .iter()
            .map(|local_model| {
                global_model.0 = global_model
                    .0
                    .iter()
                    .zip(&local_model.0)
                    .map(|(w1, w2)| w1.add(w2.mul(&stakes)))
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();

        Ok(global_model)
    }
}
