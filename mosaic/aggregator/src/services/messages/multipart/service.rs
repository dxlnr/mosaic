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
use mosaic_core::{
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