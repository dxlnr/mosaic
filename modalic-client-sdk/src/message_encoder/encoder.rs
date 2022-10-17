use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Chunker;
use modalic_core::{
    crypto::{PublicEncryptKey, SecretSigningKey, SigningKeyPair},
    message::{Chunk, Message, Payload, Tag, ToBytes},
};

/// An encoder for multipart messages. It implements
/// `Iterator<Item=Vec<u8>>`, which yields message parts ready to be
/// sent over the wire.
#[derive(Serialize, Deserialize, Debug)]
pub struct MultipartEncoder {
    keys: SigningKeyPair,
    /// The coordinator public key. It should be the key used to
    /// encrypt the message.
    coordinator_pk: PublicEncryptKey,
    /// Serialized message payload.
    data: Vec<u8>,
    /// Next chunk ID to be produced by the iterator
    id: u16,
    /// Message tag
    tag: Tag,
    /// The maximum size allowed for the payload. `self.data` is split
    /// in chunks of this size.
    payload_size: usize,
    /// A random ID common to all the message chunks.
    message_id: u16,
}

/// Overhead induced by wrapping the data in [`Payload::Chunk`]
pub const CHUNK_OVERHEAD: usize = 8;
pub const MIN_PAYLOAD_SIZE: usize = CHUNK_OVERHEAD + 1;

impl Iterator for MultipartEncoder {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let chunker = Chunker::new(&self.data, self.payload_size - CHUNK_OVERHEAD);

        if self.id as usize >= chunker.nb_chunks() {
            return None;
        }

        let chunk = Chunk {
            id: self.id,
            message_id: self.message_id,
            last: self.id as usize == chunker.nb_chunks() - 1,
            data: chunker.get_chunk(self.id as usize).to_vec(),
        };
        self.id += 1;

        let message = Message {
            // The signature is computed when serializing the message
            signature: None,
            participant_pk: self.keys.public,
            is_multipart: true,
            tag: self.tag,
            payload: Payload::Chunk(chunk),
            coordinator_pk: self.coordinator_pk,
        };
        let data = serialize_message(&message, &self.keys.secret);
        Some(data)
    }
}

/// An encoder for a [`Payload`] representing a sum, update or sum2
/// message. If the [`Payload`] is small enough, a [`Message`] header
/// is added, and the message is serialized and signed. If
/// the [`Payload`] is too large to fit in a single message, it is
/// split in chunks which are also serialized and signed.
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageEncoder {
    /// Encoder for a payload that fits in a single message.
    Simple(Option<Vec<u8>>),
    /// Encoder for a large payload that needs to be split in several
    /// parts.
    Multipart(MultipartEncoder),
}

impl Iterator for MessageEncoder {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MessageEncoder::Simple(ref mut data) => data.take(),
            MessageEncoder::Multipart(ref mut multipart_encoder) => multipart_encoder.next(),
        }
    }
}

#[derive(Error, Debug)]
pub enum InvalidEncodingInput {
    #[error("only sum, update, and sum2 messages can be encoded")]
    Payload,
    #[error("the max payload size is too small")]
    PayloadSize,
}

impl MessageEncoder {
    // NOTE: the only reason we need to consume the payload is because creating the Message
    // consumes it.
    /// Create a new encoder for the given payload. The `participant`
    /// is used to sign the message(s). If the serialized payload is
    /// larger than `max_payload_size`, the message will we split in
    /// multiple chunks. If `max_payload_size` is `0`, the message
    /// will not be split.
    ///
    /// # Errors
    ///
    /// An [`InvalidEncodingInput`] error is returned when `payload` is of
    /// type [`Payload::Chunk`]. Only [`Payload::Sum`],
    /// [`Payload::Update`], [`Payload::Sum2`] are accepted.
    pub fn new(
        keys: SigningKeyPair,
        payload: Payload,
        coordinator_pk: PublicEncryptKey,
        max_payload_size: usize,
    ) -> Result<Self, InvalidEncodingInput> {
        // Reject payloads of type Payload::Chunk. It is the job of the encoder to produce those if
        // the payload is deemed to big to be sent in a single message
        if payload.is_chunk() {
            return Err(InvalidEncodingInput::Payload);
        }

        if max_payload_size != 0 && max_payload_size <= MIN_PAYLOAD_SIZE {
            return Err(InvalidEncodingInput::PayloadSize);
        }

        println!("NEWWWWWWWWWWWWWWWWW MESSAGE: \naggr pk: {:?}", &coordinator_pk);

        if max_payload_size != 0 && payload.buffer_length() > max_payload_size {
            Ok(Self::new_multipart(
                keys,
                coordinator_pk,
                payload,
                max_payload_size,
            ))
        } else {
            Ok(Self::new_simple(keys, coordinator_pk, payload))
        }
    }

    fn new_simple(
        keys: SigningKeyPair,
        coordinator_pk: PublicEncryptKey,
        payload: Payload,
    ) -> Self {
        let message = Message {
            // The signature is computed when serializing the message
            signature: None,
            participant_pk: keys.public,
            is_multipart: false,
            coordinator_pk,
            tag: Self::get_tag_from_payload(&payload),
            payload,
        };
        let data = serialize_message(&message, &keys.secret);
        Self::Simple(Some(data))
    }

    fn new_multipart(
        keys: SigningKeyPair,
        coordinator_pk: PublicEncryptKey,
        payload: Payload,
        payload_size: usize,
    ) -> Self {
        let tag = Self::get_tag_from_payload(&payload);
        let mut data = vec![0; payload.buffer_length()];
        payload.to_bytes(&mut data);
        Self::Multipart(MultipartEncoder {
            keys,
            data,
            id: 0,
            tag,
            coordinator_pk,
            payload_size,
            message_id: rand::random::<u16>(),
        })
    }

    fn get_tag_from_payload(payload: &Payload) -> Tag {
        match payload {
            Payload::Sum(_) => Tag::Sum,
            Payload::Update(_) => Tag::Update,
            Payload::Sum2(_) => Tag::Sum2,
            Payload::Chunk(_) => panic!("no tag associated to Payload::Chunk"),
        }
    }
}

fn serialize_message(message: &Message, sk: &SecretSigningKey) -> Vec<u8> {
    let mut buf = vec![0; message.buffer_length()];
    message.to_bytes(&mut buf, sk);
    buf
}