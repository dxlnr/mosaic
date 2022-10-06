use std::{
    collections::{BTreeMap, HashMap},
    task::Poll,
};

use futures::{
    future::{self, Ready},
    task::Context,
};
use tower::Service;
use tracing::{debug, trace, warn};

use crate::services::messages::{multipart::buffer::MultipartMessageBuffer, ServiceError};
use xaynet_core::{
    crypto::{PublicEncryptKey, PublicSigningKey},
    message::{Chunk, DecodeError, FromBytes, Message, Payload, Sum, Sum2, Tag, Update},
};

/// A `MessageBuilder` stores chunks of a multipart message. Once it
/// has all the chunks, it can be consumed and turned into a
/// full-blown [`Message`] (see [`into_message()`]).
///
/// [`into_message()`]: MessageBuilder::into_message
#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct MessageBuilder {
    /// Public key of the participant sending the message
    participant_pk: PublicSigningKey,
    /// Public key of the coordinator
    coordinator_pk: PublicEncryptKey,
    /// Message type
    tag: Tag,
    /// The ID of the last chunk is actually the total number of
    /// chunks this message is made of.
    last_chunk_id: Option<u16>,
    /// Chunks, ordered by ID
    data: BTreeMap<u16, Vec<u8>>,
}

impl MessageBuilder {
    /// Create a new [`MessageBuilder`] that contains no chunk.
    fn new(tag: Tag, participant_pk: PublicSigningKey, coordinator_pk: PublicEncryptKey) -> Self {
        MessageBuilder {
            tag,
            participant_pk,
            coordinator_pk,
            data: BTreeMap::new(),
            last_chunk_id: None,
        }
    }

    /// Return `true` if the message is complete, _i.e._ if the
    /// builder holds all the chunks.
    fn has_all_chunks(&self) -> bool {
        self.last_chunk_id
            .map(|last_chunk_id| {
                // The IDs start at 0, hence the + 1
                self.data.len() >= (last_chunk_id as usize + 1)
            })
            .unwrap_or(false)
    }

    /// Add a chunk.
    fn add_chunk(&mut self, chunk: Chunk) {
        let Chunk { id, last, data, .. } = chunk;
        if last {
            self.last_chunk_id = Some(id);
        }
        self.data.insert(id, data);
    }

    /// Aggregate all the chunks. This method should only be called
    /// when all the chunks are here, otherwise the aggregated message
    /// will be invalid.
    fn into_message(self) -> Result<Message, DecodeError> {
        let mut bytes = MultipartMessageBuffer::from(self.data);
        let payload = match self.tag {
            Tag::Sum => Sum::from_byte_stream(&mut bytes).map(Into::into)?,
            Tag::Update => Update::from_byte_stream(&mut bytes).map(Into::into)?,
            Tag::Sum2 => Sum2::from_byte_stream(&mut bytes).map(Into::into)?,
        };
        let message = Message {
            signature: None,
            participant_pk: self.participant_pk,
            coordinator_pk: self.coordinator_pk,
            tag: self.tag,
            is_multipart: false,
            payload,
        };
        Ok(message)
    }
}

/// [`MessageId`] uniquely identifies a multipart message by its ID
/// (which uniquely identify a message _for a given participant_), and
/// the participant public key.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct MessageId {
    message_id: u16,
    participant_pk: PublicSigningKey,
}

/// A service that handles multipart messages.
pub struct MultipartHandler {
    message_builders: HashMap<MessageId, MessageBuilder>,
}

impl MultipartHandler {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            message_builders: HashMap::new(),
        }
    }
}

impl Service<Message> for MultipartHandler {
    type Response = Option<Message>;
    type Error = ServiceError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, message: Message) -> Self::Future {
        // If the message doesn't have the multipart flag, this
        // service has nothing to do with it.
        if !message.is_multipart {
            trace!("message is not multipart, nothing to do");
            return ready_ok(Some(message));
        }

        debug!("handling multipart message");
        if let Message {
            tag,
            participant_pk,
            coordinator_pk,
            payload: Payload::Chunk(chunk),
            ..
        } = message
        {
            let id = MessageId {
                message_id: chunk.message_id,
                participant_pk,
            };
            // If we don't have a partial message for this ID, create
            // an empty one.
            let mp_message = self.message_builders.entry(id.clone()).or_insert_with(|| {
                debug!("new multipart message (id = {})", id.message_id);
                MessageBuilder::new(tag, participant_pk, coordinator_pk)
            });
            // Add the chunk to the partial message
            mp_message.add_chunk(chunk);

            // Check if the message is complete, and if so parse it
            // and return it
            if mp_message.has_all_chunks() {
                debug!("received the final message chunk, now parsing the full message");
                // This entry exists, because `mp_message` above
                // refers to it, so it's ok to unwrap.
                match self.message_builders.remove(&id).unwrap().into_message() {
                    Ok(message) => {
                        debug!("multipart message succesfully parsed");
                        ready_ok(Some(message))
                    }
                    Err(e) => {
                        warn!("invalid multipart message: {}", e);
                        ready_err(ServiceError::Parsing(e))
                    }
                }
            } else {
                ready_ok(None)
            }
        } else {
            // This cannot happen, because parsing have fail
            panic!("multipart flag is set but payload is not a multipart message");
        }
    }
}

fn ready_ok<T, E>(t: T) -> Ready<Result<T, E>> {
    future::ready(Ok(t))
}

fn ready_err<T, E>(e: E) -> Ready<Result<T, E>> {
    future::ready(Err(e))
}

#[cfg(test)]
mod tests {
    use std::iter;

    use tokio_test::assert_ready;
    use tower_test::mock::Spawn;
    use xaynet_core::crypto::{ByteObject, PublicEncryptKey, Signature};

    use super::*;

    fn spawn_svc() -> Spawn<MultipartHandler> {
        Spawn::new(MultipartHandler::new())
    }

    fn sum() -> (Vec<u8>, Sum) {
        let mut start_byte: u8 = 0xff;
        let f = move || {
            start_byte = start_byte.wrapping_add(1) & 0b_0001_1111;
            Some(start_byte)
        };
        let bytes: Vec<u8> = iter::from_fn(f)
            .take(PublicEncryptKey::LENGTH + Signature::LENGTH)
            .collect();

        let sum = Sum {
            sum_signature: Signature::from_slice(&bytes[..Signature::LENGTH]).unwrap(),
            ephm_pk: PublicEncryptKey::from_slice(&bytes[Signature::LENGTH..]).unwrap(),
        };
        (bytes, sum)
    }

    fn message_builder() -> MessageBuilder {
        let participant_pk = PublicSigningKey::zeroed();
        let coordinator_pk = PublicEncryptKey::zeroed();
        let tag = Tag::Sum;
        MessageBuilder::new(tag, participant_pk, coordinator_pk)
    }

    fn chunks(mut data: Vec<u8>) -> (Chunk, Chunk, Chunk, Chunk, Chunk) {
        // Chunk 1: 1 byte
        // Chunk 2: 2 bytes
        // Chunk 3: 3 bytes
        // Chunk 4: 4 bytes
        // Chunk 5: 96 - (1 + 2 + 3 + 4) = 86 bytes

        assert_eq!(data.len(), 96);

        // 96 - 10 = 86, remains 10
        let data5 = data.split_off(10);
        assert_eq!(data5.len(), 86);
        assert_eq!(data.len(), 10);

        // 10 - 6 = 4, remains 6
        let data4 = data.split_off(6);
        assert_eq!(data4.len(), 4);
        assert_eq!(data.len(), 6);

        // 6 - 3 = 3, remains 3
        let data3 = data.split_off(3);
        assert_eq!(data3.len(), 3);
        assert_eq!(data.len(), 3);

        // 3 - 1 = 2, remains 1
        let data2 = data.split_off(1);
        assert_eq!(data2.len(), 2);
        assert_eq!(data.len(), 1);

        let chunk1 = Chunk {
            id: 0,
            message_id: 1234,
            last: false,
            data,
        };
        let chunk2 = Chunk {
            id: 1,
            message_id: 1234,
            last: false,
            data: data2,
        };
        let chunk3 = Chunk {
            id: 2,
            message_id: 1234,
            last: false,
            data: data3,
        };
        let chunk4 = Chunk {
            id: 3,
            message_id: 1234,
            last: false,
            data: data4,
        };
        let chunk5 = Chunk {
            id: 4,
            message_id: 1234,
            last: true,
            data: data5,
        };
        (chunk1, chunk2, chunk3, chunk4, chunk5)
    }

    #[test]
    fn test_message_builder_in_order() {
        let mut msg = message_builder();
        let (data, sum) = sum();
        let (c1, c2, c3, c4, c5) = chunks(data);

        assert!(msg.data.is_empty());
        assert!(msg.last_chunk_id.is_none());

        msg.add_chunk(c1);
        assert_eq!(msg.data.len(), 1);
        assert!(msg.last_chunk_id.is_none());
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c2);
        assert_eq!(msg.data.len(), 2);
        assert!(msg.last_chunk_id.is_none());
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c3);
        assert_eq!(msg.data.len(), 3);
        assert!(msg.last_chunk_id.is_none());
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c4);
        assert_eq!(msg.data.len(), 4);
        assert!(msg.last_chunk_id.is_none());
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c5);
        assert_eq!(msg.data.len(), 5);
        assert_eq!(msg.last_chunk_id, Some(4));
        assert!(msg.has_all_chunks());

        let actual = msg.into_message().unwrap();
        let expected =
            Message::new_sum(PublicSigningKey::zeroed(), PublicEncryptKey::zeroed(), sum);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_message_builder_out_of_order() {
        let mut msg = message_builder();
        let (data, sum) = sum();
        let (c1, c2, c3, c4, c5) = chunks(data);

        assert!(msg.data.is_empty());
        assert!(msg.last_chunk_id.is_none());

        msg.add_chunk(c3);
        assert_eq!(msg.data.len(), 1);
        assert!(msg.last_chunk_id.is_none());
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c1);
        assert_eq!(msg.data.len(), 2);
        assert!(msg.last_chunk_id.is_none());
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c5);
        assert_eq!(msg.data.len(), 3);
        assert_eq!(msg.last_chunk_id, Some(4));
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c2);
        assert_eq!(msg.data.len(), 4);
        assert_eq!(msg.last_chunk_id, Some(4));
        assert!(!msg.has_all_chunks());

        msg.add_chunk(c4);
        assert_eq!(msg.data.len(), 5);
        assert_eq!(msg.last_chunk_id, Some(4));
        assert!(msg.has_all_chunks());

        let actual = msg.into_message().unwrap();
        let expected =
            Message::new_sum(PublicSigningKey::zeroed(), PublicEncryptKey::zeroed(), sum);

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn message_handler() {
        let mut task = spawn_svc();
        assert_ready!(task.poll_ready()).unwrap();

        let coordinator_pk =
            PublicEncryptKey::from_slice(&[0x00; PublicSigningKey::LENGTH]).unwrap();

        // The payload of the message (and therefore the chunks) will
        // be the same for the two participants. What must differ is
        // the participant public key in the header.
        let (data, sum) = sum();
        let (c1, c2, c3, c4, c5) = chunks(data.clone());

        // A signing key that identifies a first faked participant.
        let pk1 = PublicSigningKey::from_slice(&[0x11; PublicSigningKey::LENGTH]).unwrap();
        // message ID for the message from our fake participant identified by `pk1`
        let message_id1 = MessageId {
            message_id: 1234,
            participant_pk: pk1,
        };
        // function that take a data chunk and create Chunk message
        // with `pk1` as participant public key in the header
        let make_message1 =
            |chunk: &Chunk| Message::new_multipart(pk1, coordinator_pk, chunk.clone(), Tag::Sum);

        // Do the same thing to fake a second participant: generate a
        // public key, a message ID, and a function to create messages
        // originating from that participant
        let pk2 = PublicSigningKey::from_slice(&[0x22; PublicSigningKey::LENGTH]).unwrap();
        let message_id2 = MessageId {
            message_id: 1234,
            participant_pk: pk2,
        };
        let make_message2 =
            |chunk: &Chunk| Message::new_multipart(pk2, coordinator_pk, chunk.clone(), Tag::Sum);

        // Start of the actual test. Notice that we send the chunks
        // out of order.

        assert!(task.call(make_message1(&c3)).await.unwrap().is_none());
        assert_eq!(task.get_ref().message_builders.len(), 1);
        let builder = task.get_ref().message_builders.get(&message_id1).unwrap();
        assert_eq!(builder.data.len(), 1);

        assert!(task.call(make_message2(&c3)).await.unwrap().is_none());
        assert_eq!(task.get_ref().message_builders.len(), 2);
        let builder = task.get_ref().message_builders.get(&message_id2).unwrap();
        assert_eq!(builder.data.len(), 1);

        assert!(task.call(make_message1(&c5)).await.unwrap().is_none());
        assert!(task.call(make_message2(&c5)).await.unwrap().is_none());

        assert!(task.call(make_message1(&c1)).await.unwrap().is_none());
        assert!(task.call(make_message2(&c1)).await.unwrap().is_none());

        assert!(task.call(make_message1(&c4)).await.unwrap().is_none());
        assert!(task.call(make_message2(&c4)).await.unwrap().is_none());

        let builder = task.get_ref().message_builders.get(&message_id1).unwrap();
        assert_eq!(builder.data.len(), 4);

        let builder = task.get_ref().message_builders.get(&message_id2).unwrap();
        assert_eq!(builder.data.len(), 4);

        let res1 = task.call(make_message1(&c2)).await.unwrap().unwrap();
        let res2 = task.call(make_message2(&c2)).await.unwrap().unwrap();

        assert!(task.get_ref().message_builders.get(&message_id1).is_none());
        assert!(task.get_ref().message_builders.get(&message_id2).is_none());

        assert_eq!(res1, Message::new_sum(pk1, coordinator_pk, sum.clone()));
        assert_eq!(res2, Message::new_sum(pk2, coordinator_pk, sum.clone()));
    }
}
