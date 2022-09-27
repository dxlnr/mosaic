use serde::{de, ser};
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum SerdeError {
    Message(String),
}

impl ser::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string())
    }
}

impl de::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string())
    }
}

impl Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerdeError::Message(msg) => write!(f, "{}", msg),
            _ => unimplemented!(),
        }
    }
}
impl std::error::Error for SerdeError {}

pub struct TensorStoreBuffer {
    _inner: Vec<u8>,
}
