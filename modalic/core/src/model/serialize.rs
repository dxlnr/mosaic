use anyhow::Context;

use crate::{
    message::{
        traits::{FromBytes, ToBytes},
        DecodeError,
    },
};

use super::ModelObject;


/// A buffer for serialized mask objects.
pub struct ModelObjectBuffer<T> {
    inner: T,
}

impl<T: AsRef<[u8]>> ModelObjectBuffer<T> {
    /// Creates a new buffer from `bytes`.
    ///
    /// # Errors
    /// Fails if the `bytes` don't conform to the required buffer length for mask objects.
    pub fn new(bytes: T) -> Result<Self, DecodeError> {
        let buffer = Self { inner: bytes };
        buffer
            .check_buffer_length()
            .context("invalid model")?;
        Ok(buffer)
    }

    /// Creates a new buffer from `bytes`.
    pub fn new_unchecked(bytes: T) -> Self {
        Self { inner: bytes }
    }

    /// Checks if this buffer conforms to the required buffer length for mask objects.
    ///
    /// # Errors
    /// Fails if the buffer is too small.
    pub fn check_buffer_length(&self) -> Result<(), DecodeError> {
        let inner = self.inner.as_ref();
        // check length of vect field
        ModelObjectBuffer::new(inner).context("invalid model")?;
        Ok(())
    }

    /// Gets the expected number of bytes of this buffer.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn len(&self) -> usize {
        todo!()
    }
}

impl ToBytes for ModelObject {
    fn buffer_length(&self) -> usize {
        todo!()
    }

    fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(&self, buffer: &mut T) {
        todo!()
    }
}

impl FromBytes for ModelObject {
    fn from_byte_slice<T: AsRef<[u8]>>(buffer: &T) -> Result<Self, DecodeError> {
        todo!()
    }

    fn from_byte_stream<I: Iterator<Item = u8> + ExactSizeIterator>(
        iter: &mut I,
    ) -> Result<Self, DecodeError> {
        todo!()
    }
}