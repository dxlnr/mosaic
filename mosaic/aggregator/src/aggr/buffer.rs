//! The buffer can be implemented by using a Trusted Execution Environment (TEE) or through
//! a cryptographic algorithm.
//!
// use std::collections::HashMap;

use crate::state_engine::states::MessageCounter;

#[cfg(feature = "secure")]
use mosaic_core::{
    SeedDict,
    mask::MaskObject,
    model::ModelObject,
};

#[cfg(not(feature = "secure"))]
use mosaic_core::model::Model;

#[cfg(not(feature = "secure"))]
#[derive(Debug, Default, Clone)]
pub struct FedBuffer {
    /// [`MessageCounter`]
    pub counter: MessageCounter,
    /// Buffered [`MaskObject`].
    pub local_models: Vec<Model>,
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
