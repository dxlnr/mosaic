//! Serialization of masked objects.
//!
pub(crate) mod unit;
pub(crate) mod vect;

use anyhow::Context;

use crate::{
    mask::object::{
        serialization::{unit::MaskUnitBuffer, vect::MaskVectBuffer},
        MaskObject, MaskUnit, MaskVect,
    },
    message::{
        traits::{FromBytes, ToBytes},
        DecodeError,
    },
};

// target dependent maximum number of mask object elements
#[cfg(target_pointer_width = "16")]
const MAX_NB: u32 = u16::MAX as u32;

/// A buffer for serialized mask objects.
pub struct MaskObjectBuffer<T> {
    inner: T,
}

impl<T: AsRef<[u8]>> MaskObjectBuffer<T> {
    /// Creates a new buffer from `bytes`.
    ///
    /// # Errors
    /// Fails if the `bytes` don't conform to the required buffer length for mask objects.
    pub fn new(bytes: T) -> Result<Self, DecodeError> {
        let buffer = Self { inner: bytes };
        buffer
            .check_buffer_length()
            .context("not a valid mask object")?;
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
        MaskVectBuffer::new(inner).context("invalid vector field")?;
        // check length of unit field
        MaskUnitBuffer::new(&inner[self.unit_offset()..]).context("invalid unit field")?;
        Ok(())
    }

    /// Gets the vector part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn vect(&self) -> &[u8] {
        let len = self.unit_offset();
        &self.inner.as_ref()[0..len]
    }

    /// Gets the offset of the unit field.
    pub fn unit_offset(&self) -> usize {
        let vect_buf = MaskVectBuffer::new_unchecked(self.inner.as_ref());
        vect_buf.len()
    }

    /// Gets the unit part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn unit(&self) -> &[u8] {
        let offset = self.unit_offset();
        &self.inner.as_ref()[offset..]
    }

    #[allow(dead_code)]
    /// Gets the expected number of bytes of this buffer.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn len(&self) -> usize {
        let unit_offset = self.unit_offset();
        let unit_buf = MaskUnitBuffer::new_unchecked(&self.inner.as_ref()[unit_offset..]);
        unit_offset + unit_buf.len()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> MaskObjectBuffer<T> {
    /// Gets the vector part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn vect_mut(&mut self) -> &mut [u8] {
        self.inner.as_mut()
    }

    /// Gets the unit part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn unit_mut(&mut self) -> &mut [u8] {
        let offset = self.unit_offset();
        &mut self.inner.as_mut()[offset..]
    }
}

impl ToBytes for MaskObject {
    fn buffer_length(&self) -> usize {
        self.vect.buffer_length() + self.unit.buffer_length()
    }

    fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(&self, buffer: &mut T) {
        let mut writer = MaskObjectBuffer::new_unchecked(buffer.as_mut());
        self.vect.to_bytes(&mut writer.vect_mut());
        self.unit.to_bytes(&mut writer.unit_mut());
    }
}

impl FromBytes for MaskObject {
    fn from_byte_slice<T: AsRef<[u8]>>(buffer: &T) -> Result<Self, DecodeError> {
        let reader = MaskObjectBuffer::new(buffer.as_ref())?;
        let vect = MaskVect::from_byte_slice(&reader.vect()).context("invalid vector part")?;
        let unit = MaskUnit::from_byte_slice(&reader.unit()).context("invalid unit part")?;
        Ok(Self { vect, unit })
    }

    fn from_byte_stream<I: Iterator<Item = u8> + ExactSizeIterator>(
        iter: &mut I,
    ) -> Result<Self, DecodeError> {
        let vect = MaskVect::from_byte_stream(iter).context("invalid vector part")?;
        let unit = MaskUnit::from_byte_stream(iter).context("invalid unit part")?;
        Ok(Self { vect, unit })
    }
}