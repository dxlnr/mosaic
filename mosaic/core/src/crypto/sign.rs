//! Wrappers around some of the [sodiumoxide] signing primitives.
//!
//! See the [crypto module] documentation since this is a private module anyways.
//!
//! [sodiumoxide]: https://docs.rs/sodiumoxide/
//! [crypto module]: crate::crypto

use std::convert::TryInto;

use derive_more::{AsMut, AsRef, From};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{hash::sha256, sign};

use super::ByteObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A `Ed25519` key pair for signatures.
pub struct SigningKeyPair {
    /// The `Ed25519` public key.
    pub public: PublicSigningKey,
    /// The `Ed25519` secret key.
    pub secret: SecretSigningKey,
}

impl SigningKeyPair {
    /// Generates a new random `Ed25519` key pair for signing.
    pub fn generate() -> Self {
        let (pk, sk) = sign::gen_keypair();
        Self {
            public: PublicSigningKey(pk),
            secret: SecretSigningKey(sk),
        }
    }

//     pub fn derive_from_seed(seed: &SigningKeySeed) -> Self {
//         let (pk, sk) = seed.derive_signing_key_pair();
//         Self {
//             public: pk,
//             secret: sk,
//         }
//     }
}

#[derive(
    AsRef,
    AsMut,
    From,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    Ord,
    PartialEq,
    Copy,
    Clone,
    PartialOrd,
    Debug,
)]
/// An `Ed25519` public key for signatures.
pub struct PublicSigningKey(sign::PublicKey);

impl PublicSigningKey {
    /// Verifies the signature `s` against the message `m` and this public key.
    ///
    /// Returns `true` if the signature is valid and `false` otherwise.
    pub fn verify_detached(&self, s: &Signature, m: &[u8]) -> bool {
        sign::verify_detached(s.as_ref(), m, self.as_ref())
    }
}

impl ByteObject for PublicSigningKey {
    const LENGTH: usize = sign::PUBLICKEYBYTES;

    fn zeroed() -> Self {
        Self(sign::PublicKey([0_u8; sign::PUBLICKEYBYTES]))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn from_slice(bytes: &[u8]) -> Option<Self> {
        sign::PublicKey::from_slice(bytes).map(Self)
    }
}

#[derive(AsRef, AsMut, From, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
/// An `Ed25519` secret key for signatures.
///
/// When this goes out of scope, its contents will be zeroed out.
pub struct SecretSigningKey(sign::SecretKey);

impl SecretSigningKey {
    /// Signs a message `m` with this secret key.
    pub fn sign_detached(&self, m: &[u8]) -> Signature {
        sign::sign_detached(m, self.as_ref()).into()
    }

    /// Computes the corresponding public key for this secret key.
    pub fn public_key(&self) -> PublicSigningKey {
        PublicSigningKey(self.0.public_key())
    }
}

impl ByteObject for SecretSigningKey {
    const LENGTH: usize = sign::SECRETKEYBYTES;

    fn zeroed() -> Self {
        Self(sign::SecretKey([0_u8; Self::LENGTH]))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn from_slice(bytes: &[u8]) -> Option<Self> {
        sign::SecretKey::from_slice(bytes).map(Self)
    }
}

#[derive(AsRef, AsMut, From, Eq, PartialEq, Copy, Clone, Debug)]
/// An `Ed25519` signature detached from its message.
pub struct Signature(sign::Signature);

mod manually_derive_serde_for_signature {
    //! TODO:
    //! remove this if sodiumoxide decides to reintroduce serialization of signatures
    //! <https://github.com/sodiumoxide/sodiumoxide/pull/434>

    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    use crate::crypto::{sign::Signature, ByteObject};

    impl Serialize for Signature {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.as_slice().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Signature {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes = <&[u8] as Deserialize>::deserialize(deserializer)?;
            Self::from_slice(bytes).ok_or_else(|| {
                D::Error::custom(format!(
                    "invalid length {}, expected {}",
                    bytes.len(),
                    Self::LENGTH,
                ))
            })
        }
    }
}

impl ByteObject for Signature {
    const LENGTH: usize = sign::SIGNATUREBYTES;

    fn zeroed() -> Self {
        Self(sign::ed25519::Signature::new([0_u8; Self::LENGTH]))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn from_slice(bytes: &[u8]) -> Option<Self> {
        bytes.try_into().ok().map(Self)
    }
}
