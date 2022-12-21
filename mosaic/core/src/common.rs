use serde::{Deserialize, Serialize};
use sodiumoxide::{self, crypto::box_};

#[cfg(feature = "secure")]
use crate::mask::MaskConfigPair;
#[cfg(not(feature = "secure"))]
use crate::model::ModelConfig;

use crate::{crypto::ByteObject, CoordinatorPublicKey};

/// The round parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoundParameters {
    /// The public key of the coordinator used for encryption.
    pub pk: CoordinatorPublicKey,
    // /// Fraction of participants to be selected for the sum task.
    // pub sum: f64,
    // /// Fraction of participants to be selected for the update task.
    // pub update: f64,
    /// The random round seed.
    pub seed: RoundSeed,
    #[cfg(not(feature = "secure"))]
    /// [`ModelConfig`]
    pub model_config: ModelConfig,
    #[cfg(feature = "secure")]
    /// The masking configuration
    pub mask_config: MaskConfigPair,
    // /// The length of the model.
    // pub model_length: usize,
    /// Sets the amount of participants in each iteration.
    pub per_round_participants: u32,
    /// Defines the number of global epochs.
    pub training_rounds: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// A seed for a round.
pub struct RoundSeed(box_::Seed);

impl ByteObject for RoundSeed {
    const LENGTH: usize = box_::SEEDBYTES;

    /// Creates a round seed from a slice of bytes.
    ///
    /// # Errors
    /// Fails if the length of the input is invalid.
    fn from_slice(bytes: &[u8]) -> Option<Self> {
        box_::Seed::from_slice(bytes).map(Self)
    }

    /// Creates a round seed initialized to zero.
    fn zeroed() -> Self {
        Self(box_::Seed([0_u8; Self::LENGTH]))
    }

    /// Gets the round seed as a slice.
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
}
