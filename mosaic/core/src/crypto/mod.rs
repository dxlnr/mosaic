//! Wrappers around some of the [sodiumoxide] crypto primitives.
//!
pub(crate) mod sign;

pub use self::{
    sign::{PublicSigningKey, SecretSigningKey, Signature, SigningKeyPair},
};

/// An interface for slicing into cryptographic byte objects.
pub trait ByteObject: Sized {
    /// Length in bytes of this object
    const LENGTH: usize;

    /// Creates a new object with all the bytes initialized to `0`.
    fn zeroed() -> Self;

    /// Gets the object byte representation.
    fn as_slice(&self) -> &[u8];

    /// Creates an object from the given buffer.
    ///
    /// # Errors
    /// Returns `None` if the length of the byte-slice isn't equal to the length of the object.
    fn from_slice(bytes: &[u8]) -> Option<Self>;

    /// Creates an object from the given buffer.
    ///
    /// # Panics
    /// Panics if the length of the byte-slice isn't equal to the length of the object.
    fn from_slice_unchecked(bytes: &[u8]) -> Self {
        Self::from_slice(bytes).unwrap()
    }

    /// Generates an object with random bytes
    fn generate() -> Self {
        // safe unwrap: length of slice is guaranteed by constants
        Self::from_slice_unchecked(sodiumoxide::randombytes::randombytes(Self::LENGTH).as_slice())
    }

    /// A helper for instantiating an object filled with the given value
    fn fill_with(value: u8) -> Self {
        Self::from_slice_unchecked(&vec![value; Self::LENGTH])
    }
}