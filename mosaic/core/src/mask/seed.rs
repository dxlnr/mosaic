//! Mask seed and mask generation.
//!
use derive_more::{AsMut, AsRef};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
// use thiserror::Error;

use crate::crypto::ByteObject;

#[derive(AsRef, AsMut, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// A seed to generate a mask.
///
/// When this goes out of scope, its contents will be zeroed out.
pub struct MaskSeed(box_::Seed);

impl ByteObject for MaskSeed {
    const LENGTH: usize = box_::SEEDBYTES;

    fn from_slice(bytes: &[u8]) -> Option<Self> {
        box_::Seed::from_slice(bytes).map(Self)
    }

    fn zeroed() -> Self {
        Self(box_::Seed([0_u8; Self::LENGTH]))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl MaskSeed {
    /// Gets this seed as an array.
    pub fn as_array(&self) -> [u8; Self::LENGTH] {
        (self.0).0
    }

    // /// Encrypts this seed with the given public key as an [`EncryptedMaskSeed`].
    // pub fn encrypt(&self, pk: &SumParticipantEphemeralPublicKey) -> EncryptedMaskSeed {
    //     todo!()
    // }

    // /// Derives a mask of given length from this seed wrt the masking configurations.
    // pub fn derive_mask(&self, len: usize, config: MaskConfigPair) -> MaskObject {
    //     todo!()
    // }
}