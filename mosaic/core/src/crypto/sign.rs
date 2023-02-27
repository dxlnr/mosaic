//! Wrappers around some of the [sodiumoxide] signing primitives.
//!
//! [sodiumoxide]: https://docs.rs/sodiumoxide/
//! [crypto module]: crate::crypto

use std::convert::TryInto;

use derive_more::{AsMut, AsRef, From};
use num::{
    bigint::{BigUint, ToBigInt},
    rational::Ratio,
};
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

    pub fn derive_from_seed(seed: &SigningKeySeed) -> Self {
        let (pk, sk) = seed.derive_signing_key_pair();
        Self {
            public: pk,
            secret: sk,
        }
    }
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

#[allow(deprecated)]
impl ByteObject for Signature {
    const LENGTH: usize = sign::SIGNATUREBYTES;

    fn zeroed() -> Self {
        Self(sign::Signature::new([0_u8; Self::LENGTH]))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    fn from_slice(bytes: &[u8]) -> Option<Self> {
        bytes.try_into().ok().map(Self)
    }
}

impl Signature {
    /// Computes the floating point representation of the hashed signature and ensures that it is
    /// below the given threshold:
    /// ```no_rust
    /// int(hash(signature)) / (2**hashbits - 1) <= threshold.
    /// ```
    pub fn is_eligible(&self, threshold: f64) -> bool {
        if threshold < 0_f64 {
            return false;
        } else if threshold > 1_f64 {
            return true;
        }
        // safe unwraps: `to_bigint` never fails for `BigUint`s
        let numer = BigUint::from_bytes_le(sha256::hash(self.as_slice()).as_ref())
            .to_bigint()
            .unwrap();
        let denom = BigUint::from_bytes_le([u8::MAX; sha256::DIGESTBYTES].as_ref())
            .to_bigint()
            .unwrap();
        // safe unwrap: `threshold` is guaranteed to be finite
        Ratio::new(numer, denom) <= Ratio::from_float(threshold).unwrap()
    }
}

#[derive(AsRef, AsMut, From, Serialize, Deserialize, Eq, PartialEq, Clone)]
/// A seed that can be used for `Ed25519` signing key pair generation.
///
/// When this goes out of scope, its contents will be zeroed out.
pub struct SigningKeySeed(sign::Seed);

impl SigningKeySeed {
    /// Deterministically derives a new signing key pair from this seed.
    pub fn derive_signing_key_pair(&self) -> (PublicSigningKey, SecretSigningKey) {
        let (pk, sk) = sign::keypair_from_seed(&self.0);
        (PublicSigningKey(pk), SecretSigningKey(sk))
    }
}

impl ByteObject for SigningKeySeed {
    const LENGTH: usize = sign::SEEDBYTES;

    fn from_slice(bytes: &[u8]) -> Option<Self> {
        sign::Seed::from_slice(bytes).map(Self)
    }

    fn zeroed() -> Self {
        Self(sign::Seed([0; sign::PUBLICKEYBYTES]))
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
}
