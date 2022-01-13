// use num::{bigint::BigInt, rational::Ratio};
use byteorder::{BigEndian, ByteOrder};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
// use std::slice::{Iter, IterMut};

use std::sync::Arc;

use num::{bigint::BigInt, rational::Ratio, traits::float::FloatCore};

/// Global model update event.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelUpdate {
    Invalidate,
    New(Arc<Model>),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A representation of a machine learning model as vector object.
pub struct Model(pub Vec<Vec<Ratio<BigInt>>>);
// pub struct Model(pub Vec<f64>);

impl std::convert::AsRef<Model> for Model {
    fn as_ref(&self) -> &Model {
        self
    }
}

impl Model {
    /// Returns the number of weights/parameters of a model.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    // /// Creates an iterator that yields references to the weights/parameters of a model.
    // pub fn iter(&self) -> Iter<f64> {
    //     self.0.iter()
    // }
    // // /// Get the model to send it back.
    // // pub fn get<'a>(&self) -> &'a Vec<f64> {
    // //     &self.0
    // // }
    //
    // /// Creates an iterator that yields mutable references to the weights/parameters of a model.
    // pub fn iter_mut(&mut self) -> IterMut<f64> {
    //     self.0.iter_mut()
    // }
    // /// Elementwise addition some data to the ['Model'] object.
    // pub fn add(&mut self, data: &Vec<f64>) {
    //     self.0 = self
    //         .iter()
    //         .zip(data)
    //         .map(|(s, x)| s + x)
    //         .collect::<Vec<_>>()
    //         .to_vec();
    // }
    // /// Elementwise averaging of ['Model'] object depending on the number of participants.
    // pub fn avg(&mut self, participants: &u32, round_id: &u32) {
    //     self.0 = self
    //         .iter()
    //         .map(|x| x / (*participants * *round_id) as f64)
    //         .collect::<Vec<_>>()
    //         .to_vec();
    // }
    fn from_bytes_array_f32(&mut self, bytes: &Vec<Vec<u8>>) {
        println!("{:?}", "I am in f32");
        self.0 = bytes
            .iter()
            .map(|r| {
                r.chunks(4)
                    .map(|x| Ratio::from_float(BigEndian::read_f32(&x)).unwrap())
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec()
    }
    fn from_bytes_array_f64(&mut self, bytes: &Vec<Vec<u8>>) {
        println!("{:?}", "I am in f64");
        self.0 = bytes
            .iter()
            .map(|r| {
                r.chunks(8)
                    .map(|x| Ratio::from_float(BigEndian::read_f64(&x)).unwrap())
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
    }
    pub fn conversion(&mut self, bytes: Vec<Vec<u8>>, dtype: &DataType) {
        match dtype {
            DataType::F32 => self.from_bytes_array_f32(&bytes),
            DataType::F64 => self.from_bytes_array_f64(&bytes),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
/// Data type that defines how byte stream of model is converted.
pub enum DataType {
    F32,
    F64,
}

impl TryFrom<u8> for DataType {
    type Error = ErrorKind;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(DataType::F32),
            1 => Ok(DataType::F64),
            _ => Err(ErrorKind::InvalidData),
        }
    }
}

// pub trait IntoBytes<T: 'static>: Sized {
//     /// Conversion from floating point values to Byte sequences.
//     fn into_bytes_array(&self) -> Vec<Vec<u8>>;
// }
//
// pub trait FromBytes<T: 'static>: Sized {
//     /// Conversion from Byte sequences into floating point values.
//     fn from_bytes_array(&self, bytes: &Vec<Vec<u8>>);
// }
//
// impl FromBytes<f32> for Model {
//     fn from_bytes_array(&self, bytes: &Vec<Vec<u8>>) {
//         self.0 = bytes
//             .iter()
//             .map(|r| {
//                 r.chunks(4)
//                     .map(|x| Ratio::from_float(BigEndian::read_f32(&x)).unwrap())
//                     .collect::<Vec<_>>()
//                     .to_vec()
//             })
//             .collect::<Vec<_>>()
//             .to_vec()
//     }
// }
//
// impl FromBytes<f64> for Model {
//     fn from_bytes_array(&self, bytes: &Vec<Vec<u8>>) {
//         self.0 = bytes
//             .iter()
//             .map(|r| {
//                 r.chunks(8)
//                     .map(|x| Ratio::from_float(BigEndian::read_f64(&x)).unwrap())
//                     .collect::<Vec<_>>()
//                     .to_vec()
//             })
//             .collect::<Vec<_>>()
//             .to_vec()
//     }
// }

#[macro_export]
macro_rules! from_bytes {
    ($model:expr, $msg:expr, $data_type:ty) => {
        impl $crate::FromBytes for $data_type {
            $model.from_bytes_array($msg)
        }
    };
}

// #[derive(Error, Debug)]
// #[error("Conversion of weight {weight} to primitive type {target}")]
// /// Errors related to model conversion into primitives.
// pub struct CastError {
//     weight: Ratio<BigInt>,
//     target: DataType,
// }

//
// #[derive(Clone, Error, Debug)]
// #[error("Could not convert primitive type {0:?} to weight")]
// /// Errors related to weight conversion from primitives.
// pub struct PrimitiveCastError<P: Debug>(pub(crate) P);

// pub trait IntoPrimitives<P: 'static>: Sized {
//     // fn into_primitives(self) -> Box<dyn Iterator<Item = Result<P, Infallible>>>;
//
//     fn to_primitives(&self) -> Vec<Vec<P>>;
// }
//
// impl IntoPrimitives<f32> for Model {
//     // fn into_primitives(self) -> Box<dyn Iterator<Item = Result<f32, Infallible>>> {
//     //     let iter = self.0.into_iter().map(|r| Ok(ratio_to_float::<f32>(&r)));
//     //     Box::new(iter)
//     // }
//
//     fn to_primitives(&self) -> Vec<Vec<f32>> {
//         // let custom_error = Error::new(ErrorKind::Other, "oh no!");
//         let vec = self.0.clone();
//         vec.iter()
//             .map(|r| {
//                 r.iter()
//                     .map(|x| ratio_to_float::<f32>(&x).unwrap())
//                     .collect::<Vec<_>>()
//                     .to_vec()
//             })
//             .collect::<Vec<_>>()
//             .to_vec()
//         // let iter = vec.into_iter().map(|r| Ok(ratio_to_float::<f32>(&r)));
//         // Box::new(iter)
//     }
// }

// impl IntoPrimitives<f64> for Model {
//     fn into_primitives(self) -> Box<dyn Iterator<Item = Result<f64, Infallible>>> {
//         let iter = self.0.into_iter().map(|r| Ok(ratio_to_float::<f64>(&r)));
//         Box::new(iter)
//     }
//
//     fn to_primitives(&self) -> Box<dyn Iterator<Item = Result<f64, Infallible>>> {
//         let vec = self.0.clone();
//         let iter = vec.into_iter().map(|r| Ok(ratio_to_float::<f64>(&r)));
//         Box::new(iter)
//     }
// }
/// Converts a numerical value into a primitive floating point value.
///
/// # Errors
/// Fails if the numerical value is not representable in the primitive data type.
pub(crate) fn ratio_to_float<F: FloatCore>(ratio: &Ratio<BigInt>) -> Option<F> {
    let min_value = Ratio::from_float(F::min_value()).unwrap();
    let max_value = Ratio::from_float(F::max_value()).unwrap();
    if ratio < &min_value || ratio > &max_value {
        return None;
    }

    let mut numer = ratio.numer().clone();
    let mut denom = ratio.denom().clone();
    // safe loop: terminates after at most bit-length of ratio iterations
    loop {
        if let (Some(n), Some(d)) = (F::from(numer.clone()), F::from(denom.clone())) {
            if n == F::zero() || d == F::zero() {
                break Some(F::zero());
            } else {
                let float = n / d;
                if float.is_finite() {
                    break Some(float);
                }
            }
        } else {
            numer >>= 1_usize;
            denom >>= 1_usize;
        }
    }
}
