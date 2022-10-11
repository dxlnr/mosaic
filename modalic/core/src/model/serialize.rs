use std::ops::Range;

use anyhow::Context;

use crate::{
    message::{
        traits::{FromBytes, ToBytes},
        utils::range,
        DecodeError,
    },
    model::ratio_to_bytes,
};

use super::ModelObject;

const DATA_TYPE_FIELD: usize = 0;
// const MASK_CONFIG_FIELD: Range<usize> = range(0, 1);
const NUMBERS_FIELD: Range<usize> = range(DATA_TYPE_FIELD + 1, 4);

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

    // /// Gets the expected number of bytes of this buffer.
    // ///
    // /// # Panics
    // /// May panic if this buffer is unchecked.
    // pub fn len(&self) -> usize {
    //     let config = MaskConfig::from_byte_slice(&self.config()).unwrap();
    //     let bytes_per_number = config.bytes_per_number();
    //     let data_length = self.numbers() * bytes_per_number;
    //     NUMBERS_FIELD.end + data_length
    // }

    // pub fn data(&self) -> &[u8] {
    //     &self.inner.as_ref()[1..self.len()]
    // }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ModelObjectBuffer<T> {
    /// Sets the number of serialized mask vector elements.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn set_numbers(&mut self, value: u32) {
        self.inner.as_mut()[NUMBERS_FIELD].copy_from_slice(&value.to_be_bytes());
    }
    /// Gets the unit part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn datatype_mut(&mut self) -> &mut u8 {
        &mut self.inner.as_mut()[DATA_TYPE_FIELD]
    }

    /// Gets the vector part.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn data_mut(&mut self, length: usize) -> &mut [u8] {
        &mut self.inner.as_mut()[NUMBERS_FIELD.end..length]
    }
}

impl<T: AsMut<[u8]>> ModelObjectBuffer<T> {
    /// Sets the serialized data type of the masking configuration.
    ///
    /// # Panics
    /// May panic if this buffer is unchecked.
    pub fn set_data_type(&mut self, value: u8) {
        self.inner.as_mut()[DATA_TYPE_FIELD] = value;
    }
}

impl ToBytes for ModelObject {
    fn buffer_length(&self) -> usize {
        NUMBERS_FIELD.end + self.data_type.bytes_per_number() * self.data.len()
    }

    fn to_bytes<T: AsMut<[u8]> + AsRef<[u8]>>(&self, buffer: &mut T) {
        let mut writer = ModelObjectBuffer::new_unchecked(buffer.as_mut());
        writer.set_data_type(self.data_type as u8);
        writer.set_numbers(self.data.len() as u32);

        let mut data = writer.data_mut(self.buffer_length());
        
        let bytes_per_number = self.data_type.bytes_per_number();
        
        for ratio in self.data.iter() {
            let bytes = ratio_to_bytes(ratio, self.data_type);
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

    use num::{bigint::BigInt, rational::Ratio};
    use crate::model::DataType;

    #[test]
    pub fn serialize_model_object() {

        let mut bytes = vec![0x00];
        bytes.extend(vec![
            // number of elements
            0x00, 0x00, 0x00, 0x04,  // data (1 weight => 4 bytes with f32)
            0x01, 0x00, 0x00, 0x00,  // 1
            0x02, 0x00, 0x00, 0x00,  // 2
            0x01, 0x00, 0x00, 0x00,  // 1
            0x02, 0x00, 0x00, 0x00,  // 2
        ]);

        let data = vec![
            Ratio::new(BigInt::from(1_u8), BigInt::from(1_u8)),
            Ratio::new(BigInt::from(2_u8), BigInt::from(1_u8)),
            Ratio::new(BigInt::from(1_u8), BigInt::from(1_u8)),
            Ratio::new(BigInt::from(2_u8), BigInt::from(1_u8)),
        ];

        let m_obj = ModelObject::new(data, DataType::F32);
        let mut buf = vec![0xff; m_obj.buffer_length()];
        m_obj.to_bytes(&mut buf);
    }
}