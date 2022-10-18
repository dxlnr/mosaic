//! Serialization of model configurations & meta data.
//!
//! See the [model module] documentation since this is a private module anyways.
//!
//! [model module]: crate::model

use std::convert::TryInto;

use anyhow::{anyhow, Context};

use crate::{
    message::{
        traits::{FromBytes, ToBytes},
        DecodeError,
    },
    model::config::ModelConfig,
};

const DATA_TYPE_FIELD: usize = 0;
pub(crate) const MODEL_CONFIG_BUFFER_LEN: usize = 1;

/// A buffer for serialized model configurations.
pub struct ModelConfigBuffer<T> {
    inner: T,
}

impl<T: AsRef<[u8]>> ModelConfigBuffer<T> {
    /// Creates a new buffer from `bytes`.
    ///
    /// # Errors
    /// Fails if the `bytes` don't conform to the required buffer length for model configurations.
    pub fn new(bytes: T) -> Result<Self, DecodeError> {
        let buffer = Self { inner: bytes };
        buffer
            .check_buffer_length()
            .context("not a valid ModelConfigBuffer")?;
        Ok(buffer)
    }

    /// Creates a new buffer from `bytes`.
    pub fn new_unchecked(bytes: T) -> Self {
        Self { inner: bytes }
    }

    /// Checks if this buffer conforms to the required buffer length for model configurations.
    ///
    /// # Errors
    /// Fails if the buffer is too small.
    pub fn check_buffer_length(&self) -> Result<(), DecodeError> {
        let len = self.inner.as_ref().len();
        if len < MODEL_CONFIG_BUFFER_LEN {
            return Err(anyhow!(
                "invalid buffer length: {} < {}",
                len,
                MODEL_CONFIG_BUFFER_LEN
            ));
        }
        Ok(())
    }

    /// Gets the serialized data type of the model configuration.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn data_type(&self) -> u8 {
        self.inner.as_ref()[DATA_TYPE_FIELD]
    }
}

impl<T: AsMut<[u8]>> ModelConfigBuffer<T> {
    /// Sets the serialized data type of the model configuration.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn set_data_type(&mut self, value: u8) {
        self.inner.as_mut()[DATA_TYPE_FIELD] = value;
    }
}

impl ToBytes for ModelConfig {
    fn buffer_length(&self) -> usize {
        MODEL_CONFIG_BUFFER_LEN
    }

    fn to_bytes<T: AsMut<[u8]>>(&self, buffer: &mut T) {
        let mut writer = ModelConfigBuffer::new_unchecked(buffer.as_mut());
        writer.set_data_type(self.data_type as u8);
    }
}

impl FromBytes for ModelConfig {
    fn from_byte_slice<T: AsRef<[u8]>>(buffer: &T) -> Result<Self, DecodeError> {
        let reader = ModelConfigBuffer::new(buffer.as_ref())?;
        Ok(Self {
            data_type: reader
                .data_type()
                .try_into()
                .context("invalid model configuration & meta data.")?,
        })
    }

    fn from_byte_stream<I: Iterator<Item = u8> + ExactSizeIterator>(
        iter: &mut I,
    ) -> Result<Self, DecodeError> {
        let buf: Vec<u8> = iter.take(MODEL_CONFIG_BUFFER_LEN).collect();
        Self::from_byte_slice(&buf)
    }
}
