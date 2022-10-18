//! The buffer can be implemented by using a Trusted Execution Environment (TEE) or through
//! a cryptographic algorithm.
//!
// use std::collections::HashMap;

use crate::state_engine::states::MessageCounter;

#[cfg(feature = "secure")]
use modalic_core::{
    SeedDict,
    mask::MaskObject,
    model::ModelObject,
};

#[cfg(not(feature = "secure"))]
use modalic_core::model::Model;

#[cfg(not(feature = "secure"))]
#[derive(Debug, Clone)]
pub struct FedBuffer {
    /// [`MessageCounter`]
    pub counter: MessageCounter,
    /// Buffered [`MaskObject`].
    pub local_models: Vec<Model>,
}

impl Default for FedBuffer {
    fn default() -> Self {
        Self {
            counter: MessageCounter::default(),
            local_models: Vec::new(),
        }
    }
}

#[cfg(feature = "secure")]
#[derive(Debug, Clone)]
pub struct FedBuffer {
    /// [`MessageCounter`]
    pub counter: MessageCounter,
    /// Buffered [`MaskObject`].
    pub local_models: Vec<MaskObject>,
    /// The seed dictionary which gets assembled during the update phase.
    pub seed_dict: Option<SeedDict>,
}
