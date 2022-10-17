use std::ops::Range;

use anyhow::{anyhow, Context};

use crate::{
    message::{
        traits::{FromBytes, ToBytes},
        utils::range,
        DecodeError,
    },
    model::{
        config::{serialize::MODEL_CONFIG_BUFFER_LEN, ModelConfig},
        ratio_to_bytes,
        ModelObject
    }
};

const MODEL_CONFIG_FIELD: Range<usize> = range(0, MODEL_CONFIG_BUFFER_LEN);
const MODEL_LEN_FIELD: Range<usize> = range(MODEL_CONFIG_FIELD.end, 4);

#[derive(Debug)]
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
            .context("not a valid mask vector")?;
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
        // let inner = self.inner.as_ref();
        let len = self.inner.as_ref().len();

        if len < MODEL_LEN_FIELD.end {
            return Err(anyhow!(
                "invalid buffer length: {} < {}",
                len,
                MODEL_LEN_FIELD.end
            ));
        }

        let total_expected_length = self.try_len()?;
        if len < total_expected_length {
            return Err(anyhow!(
                "invalid buffer length: expected {} bytes but buffer has only {} bytes",
                total_expected_length,
                len
            ));
        }
        Ok(())
    }

    /// Return the expected length of the underlying byte buffer,
    /// based on the masking config field of numbers field. This is
    /// similar to [`len()`] but cannot panic.
    ///
    /// [`len()`]: MaskVectBuffer::len
    fn try_len(&self) -> Result<usize, DecodeError> {
        let config =
            ModelConfig::from_byte_slice(&self.config()).context("invalid mask vector buffer")?;
        let bytes_per_number = config.bytes_per_number();
        let (data_length, overflows) = self.numbers().overflowing_mul(bytes_per_number);
        if overflows {
            return Err(anyhow!(
                "invalid MaskObject buffer: invalid masking config or numbers field"
            ));
        }
        Ok(MODEL_LEN_FIELD.end + data_length)
    }

    /// Gets the expected number of bytes of this buffer.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn len(&self) -> usize {
        let config = ModelConfig::from_byte_slice(&self.config()).unwrap();
        let bytes_per_number = config.bytes_per_number();
        let data_length = self.numbers() * bytes_per_number;
        MODEL_LEN_FIELD.end + data_length
    }

    pub fn numbers(&self) -> usize {
        // UNWRAP SAFE: the slice is exactly 4 bytes long
        let nb = u32::from_be_bytes(self.inner.as_ref()[MODEL_LEN_FIELD].try_into().unwrap());

        // smaller targets than 32 bits are currently not of interest
        #[cfg(target_pointer_width = "16")]
        if nb > MAX_NB {
            panic!("16 bit targets or smaller are currently not fully supported")
        }

        nb as usize
    }

    /// Gets the serialized model configuration.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn config(&self) -> &[u8] {
        &self.inner.as_ref()[MODEL_CONFIG_FIELD]
    }

    /// Gets the serialized model vector elements.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn data(&self) -> &[u8] {
        &self.inner.as_ref()[MODEL_LEN_FIELD.end..self.len()]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ModelObjectBuffer<T> {
    /// Sets the number of serialized mask vector elements.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn set_numbers(&mut self, value: u32) {
        self.inner.as_mut()[MODEL_LEN_FIELD].copy_from_slice(&value.to_be_bytes());
    }

    /// Gets the serialized masking configuration.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn config_mut(&mut self) -> &mut [u8] {
        &mut self.inner.as_mut()[MODEL_CONFIG_FIELD]
    }

    /// Gets the vector part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn data_mut(&mut self, length: usize) -> &mut [u8] {
        &mut self.inner.as_mut()[MODEL_LEN_FIELD.end..length]
    }
}

impl ToBytes for ModelObject {
    fn buffer_length(&self) -> usize {
        MODEL_LEN_FIELD.end + self.config.bytes_per_number() * self.data.len()
    }

    fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(&self, buffer: &mut T) {
        let mut writer = ModelObjectBuffer::new_unchecked(buffer.as_mut());
        self.config.to_bytes(&mut writer.config_mut());
        writer.set_numbers(self.data.len() as u32);

        let mut data = writer.data_mut(self.buffer_length());

        let bytes_per_number = self.config.bytes_per_number();

        for ratio in self.data.iter() {
            let bytes = ratio_to_bytes(ratio, self.config.data_type);
            data[..bytes.len()].copy_from_slice(&bytes[..]);

            for b in data.iter_mut().take(bytes_per_number).skip(bytes.len()) {
                *b = 0;
            }
            data = &mut data[bytes_per_number..];
        }
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::model::{ModelConfig, DataType};
    use num::{bigint::BigInt, rational::Ratio};

    #[test]
    pub fn serialize_model_object() {
        let mut bytes = vec![0x00];
        bytes.extend(vec![
            // number of elements
            0x00, 0x00, 0x00, 0x04, // data (1 weight => 4 bytes with f32)
            0x01, 0x00, 0x00, 0x00, // 1
            0x02, 0x00, 0x00, 0x00, // 2
            0x01, 0x00, 0x00, 0x00, // 1
            0x02, 0x00, 0x00, 0x00, // 2
        ]);

        let data = vec![
            Ratio::new(BigInt::from(1_u8), BigInt::from(1_u8)),
            Ratio::new(BigInt::from(2_u8), BigInt::from(1_u8)),
            Ratio::new(BigInt::from(1_u8), BigInt::from(1_u8)),
            Ratio::new(BigInt::from(2_u8), BigInt::from(1_u8)),
        ];

        let m_obj = ModelObject::new(data, ModelConfig { data_type: DataType::F32});
        let mut buf = vec![0xff; m_obj.buffer_length()];
        m_obj.to_bytes(&mut buf);
    }
}
