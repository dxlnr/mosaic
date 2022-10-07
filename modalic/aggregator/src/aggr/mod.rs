use serde::{Deserialize, Serialize};

use crate::settings::{
    MaskSettings,
    // ModelSettings,
};

use modalic_core::{
    common::{RoundParameters, RoundSeed},
    crypto::{ByteObject, EncryptKeyPair},
    mask::MaskConfig,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aggregator {
    /// The credentials of the coordinator.
    pub keys: EncryptKeyPair,
    /// Current progress towards an aggregation goal.
    pub round_id: u32,
    /// The [`RoundParameters`].
    pub round_params: RoundParameters,
    /// Hyperparameter comprised in [`AggrParams`].
    pub params: AggrParams,

}

impl Aggregator {
    pub fn new(mask_settings: MaskSettings) -> Self {
        let keys = EncryptKeyPair::generate();

        let round_params = RoundParameters {
            pk: keys.public,
            // sum: pet_settings.sum.prob,
            // update: pet_settings.update.prob,
            seed: RoundSeed::zeroed(),
            mask_config: MaskConfig::from(mask_settings).into(),
            // model_length: model_settings.length,
        };

        Self {
            keys,
            round_id: 0,
            round_params,
            params: AggrParams::default(),
        }
    }
}

/// Parameters necessary for performing an aggregation schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggrParams {
    /// Server-side learning rate. Defaults to 1e-1.
    pub eta: f64,
    /// Client updates are stored in a buffer. A server update only takes place
    /// once K client updates are in the buffer, where K is the size of the buffer.
    ///
    /// According to [Nguyen et al. 2021](https://arxiv.org/abs/2106.06639) k = 10 seems to be
    /// a good fit that needs no further tuning.
    pub k: u32,
}

impl AggrParams {
    /// Creates new [`AggrParams`] which allows altering the default parameters.
    pub fn new(eta: f64, k: u32) -> Self {
        Self { eta, k }
    }
}

impl Default for AggrParams {
    fn default() -> Self {
        Self { eta: 1e-1, k: 10 }
    }
}