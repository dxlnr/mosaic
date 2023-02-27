//! The messages of the protocol.
//!
//! # The update message
//! The [`Update`] message is an abstraction for the values which an update participant communicates
//! to mosaic during the update phase of the protocol. It contains the following values:
//! - The sum signature proves the ineligibility of the participant for the sum task.
//! - The update signature proves the eligibility of the participant for the update task.
//! - The masked model is the encrypted local update to the global model, which is trained on the
//!   local data of the update participant.
//! - The local seed dictionary stores the encrypted mask seed, which generates the local mask for
//!   the local model, which is encrypted by the ephemeral public keys of the sum participants.
//!
#[allow(clippy::module_inception)]
pub(crate) mod message;
pub(crate) mod payload;
pub(crate) mod traits;
pub(crate) mod utils;

pub use self::{
    message::{
        Flags, Message, MessageBuffer, Tag, HEADER_LENGTH as MESSAGE_HEADER_LENGTH, SUM_COUNT_MIN,
        UPDATE_COUNT_MIN,
    },
    payload::{
        chunk::{Chunk, ChunkBuffer},
        sum::{Sum, SumBuffer},
        sum2::{Sum2, Sum2Buffer},
        update::{Update, UpdateBuffer},
        Payload,
    },
    traits::{FromBytes, LengthValueBuffer, ToBytes},
};

/// An error that signals a failure when trying to decrypt and parse a message.
///
/// This is kept generic on purpose to not reveal to the sender what specifically failed during
/// decryption or parsing.
pub type DecodeError = anyhow::Error;
