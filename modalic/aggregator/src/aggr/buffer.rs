//! The buffer can be implemented by using a Trusted Execution Environment (TEE) or through
//! a cryptographic algorithm.
//!
// use std::collections::HashMap;

use crate::state_engine::states::MessageCounter;

use modalic_core::{
    SeedDict,
    mask::MaskObject, 
};

#[derive(Debug, Clone)]
pub struct FedBuffer {
    /// [`MessageCounter`]
    pub counter: MessageCounter,
    /// Buffered [`MaskObject`].
    pub local_models: Vec<MaskObject>,
    /// The seed dictionary which gets assembled during the update phase.
    pub seed_dict: Option<SeedDict>,
}

impl Default for FedBuffer {
    /// Creates a new default [`MessageCounter`].
    fn default() -> Self {
        Self {
            counter: MessageCounter::default(),
            local_models: Vec::new(),
            seed_dict: None,
        }
    }
}

// impl FedBuffer {
//     pub fn set_seed_dict(&mut self, k: PublicSigningKey, v: UpdateSeedDict) {
//         self.seed_dict(k, v);
//     }
// }