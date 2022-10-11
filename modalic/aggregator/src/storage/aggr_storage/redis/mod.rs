//! A Redis [`AggregatorStorage`] backend.
//!
//! # Redis Data Model
//!
//!```text
//! {
//!     // Coordinator state
//!     "coordinator_state": "...", // bincode encoded string
//!     // Sum dict
//!     "sum_dict": { // hash
//!         "SumParticipantPublicKey_1": SumParticipantEphemeralPublicKey_1,
//!         "SumParticipantPublicKey_2": SumParticipantEphemeralPublicKey_2
//!     },
//!     // Seed dict
//!     "update_participants": [ // set
//!         UpdateParticipantPublicKey_1,
//!         UpdateParticipantPublicKey_2
//!     ],
//!     "SumParticipantPublicKey_1": { // hash
//!         "UpdateParticipantPublicKey_1": EncryptedMaskSeed,
//!         "UpdateParticipantPublicKey_2": EncryptedMaskSeed
//!     },
//!     "SumParticipantPublicKey_2": {
//!         "UpdateParticipantPublicKey_1": EncryptedMaskSeed,
//!         "UpdateParticipantPublicKey_2": EncryptedMaskSeed
//!     },
//!     // Mask dict
//!     "mask_submitted": [ // set
//!         SumParticipantPublicKey_1,
//!         SumParticipantPublicKey_2
//!     ],
//!     "mask_dict": [ // sorted set
//!         (mask_object_1, 2), // (mask: bincode encoded string, score/counter: number)
//!         (mask_object_2, 1)
//!     ],
//!     "latest_global_model_id": global_model_id
//! }
//! ```

pub(in crate::storage) mod impls;

use std::collections::HashMap;

use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, IntoConnectionInfo, Pipeline, Script};
pub use redis::{RedisError, RedisResult};
use tracing::debug;

use self::impls::{
    EncryptedMaskSeedRead,
    LocalSeedDictWrite,
    MaskObjectRead,
    MaskObjectWrite,
    PublicEncryptKeyRead,
    PublicEncryptKeyWrite,
    PublicSigningKeyRead,
    PublicSigningKeyWrite,
};
use crate::{
    aggr::Aggregator,
    // state_engine::coordinator::Aggregator,
    storage::{
        AggregatorStorage,
        // AggregatorStorage,
        LocalSeedDictAdd,
        MaskScoreIncr,
        StorageError,
        StorageResult,
        SumPartAdd,
    },
};
use modalic_core::{
    mask::MaskObject,
    LocalSeedDict,
    SeedDict,
    SumDict,
    SumParticipantEphemeralPublicKey,
    SumParticipantPublicKey,
    UpdateParticipantPublicKey,
};

/// Redis client.
#[derive(Clone)]
pub struct Client {
    connection: ConnectionManager,
}

fn to_storage_err(e: RedisError) -> StorageError {
    anyhow::anyhow!(e)
}

impl Client {
    /// Creates a new Redis client.
    ///
    /// `url` to which Redis instance the client should connect to.
    /// The URL format is `redis://[<username>][:<passwd>@]<hostname>[:port][/<db>]`.
    ///
    /// The [`Client`] uses a [`ConnectionManager`] that automatically reconnects
    /// if the connection is dropped.
    pub async fn new<T: IntoConnectionInfo>(url: T) -> Result<Self, RedisError> {
        let client = redis::Client::open(url)?;
        let connection = client.get_tokio_connection_manager().await?;
        Ok(Self { connection })
    }

    async fn create_flush_dicts_pipeline(&mut self) -> RedisResult<Pipeline> {
        // https://redis.io/commands/hkeys
        // > Return value:
        //   Array reply: list of fields in the hash, or an empty list when key does not exist.
        let sum_pks: Vec<PublicSigningKeyRead> = self.connection.hkeys("sum_dict").await?;
        let mut pipe = redis::pipe();

        // https://redis.io/commands/del
        // > Return value:
        //   The number of keys that were removed.
        //
        // Returns `0` if the key does not exist.
        // We ignore the return value because we are not interested in it.

        // delete sum dict
        pipe.del("sum_dict").ignore();

        // delete seed dict
        pipe.del("update_participants").ignore();
        for sum_pk in sum_pks {
            pipe.del(sum_pk).ignore();
        }

        // delete mask dict
        pipe.del("mask_submitted").ignore();
        pipe.del("mask_dict").ignore();
        Ok(pipe)
    }
}

#[async_trait]
impl AggregatorStorage for Client {
    async fn set_coordinator_state(&mut self, state: &Aggregator) -> StorageResult<()> {
        debug!("set coordinator state");
        // https://redis.io/commands/set
        // > Set key to hold the string value. If key already holds a value,
        //   it is overwritten, regardless of its type.
        // Possible return value in our case:
        // > Simple string reply: OK if SET was executed correctly.
        self.connection
            .set("coordinator_state", state)
            .await
            .map_err(to_storage_err)
    }

    async fn coordinator_state(&mut self) -> StorageResult<Option<Aggregator>> {
        // https://redis.io/commands/get
        // > Get the value of key. If the key does not exist the special value nil is returned.
        //   An error is returned if the value stored at key is not a string, because GET only
        //   handles string values.
        // > Return value
        //   Bulk string reply: the value of key, or nil when key does not exist.
        self.connection
            .get("coordinator_state")
            .await
            .map_err(to_storage_err)
    }

    // async fn add_sum_participant(
    //     &mut self,
    //     pk: &SumParticipantPublicKey,
    //     ephm_pk: &SumParticipantEphemeralPublicKey,
    // ) -> StorageResult<SumPartAdd> {
    //     debug!("add sum participant with pk {:?}", pk);
    //     // https://redis.io/commands/hsetnx
    //     // > If field already exists, this operation has no effect.
    //     // > Return value
    //     //   Integer reply, specifically:
    //     //   1 if field is a new field in the hash and value was set.
    //     //   0 if field already exists in the hash and no operation was performed.
    //     self.connection
    //         .hset_nx(
    //             "sum_dict",
    //             PublicSigningKeyWrite::from(pk),
    //             PublicEncryptKeyWrite::from(ephm_pk),
    //         )
    //         .await
    //         .map_err(to_storage_err)
    // }

    // async fn sum_dict(&mut self) -> StorageResult<Option<SumDict>> {
    //     debug!("get sum dictionary");
    //     // https://redis.io/commands/hgetall
    //     // > Return value
    //     //   Array reply: list of fields and their values stored in the hash, or an empty
    //     //   list when key does not exist.
    //     let reply: Vec<(PublicSigningKeyRead, PublicEncryptKeyRead)> = self
    //         .connection
    //         .hgetall("sum_dict")
    //         .await
    //         .map_err(to_storage_err)?;

    //     if reply.is_empty() {
    //         return Ok(None);
    //     };

    //     let sum_dict = reply
    //         .into_iter()
    //         .map(|(pk, ephm_pk)| (pk.into(), ephm_pk.into()))
    //         .collect();

    //     Ok(Some(sum_dict))
    // }

    // async fn add_local_seed_dict(
    //     &mut self,
    //     update_pk: &UpdateParticipantPublicKey,
    //     local_seed_dict: &LocalSeedDict,
    // ) -> StorageResult<LocalSeedDictAdd> {
    //     debug!(
    //         "update seed dictionary for update participant with pk {:?}",
    //         update_pk
    //     );
    //     let script = Script::new(
    //         r#"
    //             -- lua lists (tables) start at 1
    //             local update_pk = ARGV[1]

    //             -- check if the local seed dict has the same length as the sum_dict

    //             -- KEYS is a list (table) of key value pairs ([sum_pk_1, seed_1, sum_pk_2, seed_2, ...])
    //             local seed_dict_len = #KEYS / 2
    //             local sum_dict_len = redis.call("HLEN", "sum_dict")
    //             if seed_dict_len ~= sum_dict_len then
    //                 return -1
    //             end

    //             -- check if all pks of the local seed dict exists in sum_dict
    //             for i = 1, #KEYS, 2 do
    //                 local exist_in_sum_dict = redis.call("HEXISTS", "sum_dict", KEYS[i])
    //                 if exist_in_sum_dict == 0 then
    //                     return -2
    //                 end
    //             end

    //             -- check if the update pk already exists (i.e. the local seed dict has already been submitted)
    //             local exist_in_seed_dict = redis.call("SADD", "update_participants", update_pk)
    //             -- SADD returns 0 if the key already exists
    //             if exist_in_seed_dict == 0 then
    //                 return -3
    //             end

    //             -- update the seed dict
    //             for i = 1, #KEYS, 2 do
    //                 local exist_in_update_seed_dict = redis.call("HSETNX", KEYS[i], update_pk, KEYS[i + 1])
    //                 -- HSETNX returns 0 if the update pk already exists
    //                 if exist_in_update_seed_dict == 0 then
    //                     -- This condition should never apply.
    //                     -- If this condition is true, it is an indication that the data in redis is corrupted.
    //                     return -4
    //                 end
    //             end

    //             return 0
    //         "#,
    //     );

    //     script
    //         .key(LocalSeedDictWrite::from(local_seed_dict))
    //         .arg(PublicSigningKeyWrite::from(update_pk))
    //         .invoke_async(&mut self.connection)
    //         .await
    //         .map_err(to_storage_err)
    // }

    // /// # Note
    // /// This method is **not** an atomic operation.
    // async fn seed_dict(&mut self) -> StorageResult<Option<SeedDict>> {
    //     debug!("get seed dictionary");
    //     // https://redis.io/commands/hkeys
    //     // > Return value:
    //     //   Array reply: list of fields in the hash, or an empty list when key does not exist.
    //     let sum_pks: Vec<PublicSigningKeyRead> = self.connection.hkeys("sum_dict").await?;

    //     if sum_pks.is_empty() {
    //         return Ok(None);
    //     };

    //     let mut seed_dict: SeedDict = SeedDict::new();
    //     for sum_pk in sum_pks {
    //         // https://redis.io/commands/hgetall
    //         // > Return value
    //         //   Array reply: list of fields and their values stored in the hash, or an empty
    //         //   list when key does not exist.
    //         let sum_pk_seed_dict: HashMap<PublicSigningKeyRead, EncryptedMaskSeedRead> =
    //             self.connection.hgetall(&sum_pk).await?;
    //         seed_dict.insert(
    //             sum_pk.into(),
    //             sum_pk_seed_dict
    //                 .into_iter()
    //                 .map(|(pk, seed)| (pk.into(), seed.into()))
    //                 .collect(),
    //         );
    //     }

    //     Ok(Some(seed_dict))
    // }

    // /// The maximum length of a serialized mask is 512 Megabytes.
    // async fn incr_mask_score(
    //     &mut self,
    //     sum_pk: &SumParticipantPublicKey,
    //     mask: &MaskObject,
    // ) -> StorageResult<MaskScoreIncr> {
    //     debug!("increment mask count");
    //     let script = Script::new(
    //         r#"
    //             -- lua lists (tables) start at 1
    //             local sum_pk = ARGV[1]

    //             -- check if the client participated in sum phase
    //             --
    //             -- Note: we cannot delete the sum_pk in the sum_dict because we
    //             -- need the sum_dict later to delete the seed_dict
    //             local sum_pk_exist = redis.call("HEXISTS", "sum_dict", sum_pk)
    //             if sum_pk_exist == 0 then
    //                 return -1
    //             end

    //             -- check if sum participant has not already submitted a mask
    //             local mask_already_submitted = redis.call("SADD", "mask_submitted", sum_pk)
    //             -- SADD returns 0 if the key already exists
    //             if mask_already_submitted == 0 then
    //                 return -2
    //             end

    //             redis.call("ZINCRBY", "mask_dict", 1, KEYS[1])

    //             return 0
    //         "#,
    //     );

    //     script
    //         .key(MaskObjectWrite::from(mask))
    //         .arg(PublicSigningKeyWrite::from(sum_pk))
    //         .invoke_async(&mut self.connection)
    //         .await
    //         .map_err(to_storage_err)
    // }

    // async fn best_masks(&mut self) -> StorageResult<Option<Vec<(MaskObject, u64)>>> {
    //     debug!("get best masks");
    //     // https://redis.io/commands/zrevrangebyscore
    //     // > Return value:
    //     //   Array reply: list of elements in the specified range (optionally with their scores,
    //     //   in case the WITHSCORES option is given).
    //     let reply: Vec<(MaskObjectRead, u64)> = self
    //         .connection
    //         .zrevrange_withscores("mask_dict", 0, 1)
    //         .await?;

    //     let result = match reply.is_empty() {
    //         true => None,
    //         _ => {
    //             let masks = reply
    //                 .into_iter()
    //                 .map(|(mask, count)| (mask.into(), count))
    //                 .collect();

    //             Some(masks)
    //         }
    //     };

    //     Ok(result)
    // }

    // async fn number_of_unique_masks(&mut self) -> StorageResult<u64> {
    //     debug!("get number of unique masks");
    //     // https://redis.io/commands/zcount
    //     // > Return value:
    //     //   Integer reply: the number of elements in the specified score range.
    //     self.connection
    //         .zcount("mask_dict", "-inf", "+inf")
    //         .await
    //         .map_err(to_storage_err)
    // }

    /// # Note
    /// This method is **not** an atomic operation.
    async fn delete_coordinator_data(&mut self) -> StorageResult<()> {
        debug!("flush coordinator data");
        let mut pipe = self.create_flush_dicts_pipeline().await?;
        pipe.del("coordinator_state").ignore();
        pipe.del("latest_global_model_id").ignore();
        pipe.atomic()
            .query_async(&mut self.connection)
            .await
            .map_err(to_storage_err)
    }

    /// # Note
    /// This method is **not** an atomic operation.
    async fn delete_dicts(&mut self) -> StorageResult<()> {
        debug!("flush all dictionaries");
        let mut pipe = self.create_flush_dicts_pipeline().await?;
        pipe.atomic()
            .query_async(&mut self.connection)
            .await
            .map_err(to_storage_err)
    }

    async fn set_latest_global_model_id(&mut self, global_model_id: &str) -> StorageResult<()> {
        debug!("set latest global model with id {}", global_model_id);
        // https://redis.io/commands/set
        // > Set key to hold the string value. If key already holds a value,
        //   it is overwritten, regardless of its type.
        // Possible return value in our case:
        // > Simple string reply: OK if SET was executed correctly.
        self.connection
            .set("latest_global_model_id", global_model_id)
            .await
            .map_err(to_storage_err)
    }

    async fn latest_global_model_id(&mut self) -> StorageResult<Option<String>> {
        debug!("get latest global model id");
        // https://redis.io/commands/get
        // > Get the value of key. If the key does not exist the special value nil is returned.
        //   An error is returned if the value stored at key is not a string, because GET only
        //   handles string values.
        // > Return value
        //   Bulk string reply: the value of key, or nil when key does not exist.
        self.connection
            .get("latest_global_model_id")
            .await
            .map_err(to_storage_err)
    }

    async fn is_ready(&mut self) -> StorageResult<()> {
        // https://redis.io/commands/ping
        redis::cmd("PING")
            .query_async(&mut self.connection)
            .await
            .map_err(to_storage_err)
    }
}

#[cfg(test)]
// Functions that are not needed in the state machine but handy for testing.
impl Client {
    // Removes an entry in the [`SumDict`].
    //
    // Returns [`SumDictDelete(Ok(()))`] if field was deleted or
    // [`SumDictDelete(Err(SumDictDeleteError::DoesNotExist)`] if field does not exist.
    pub async fn remove_sum_dict_entry(
        &mut self,
        pk: &SumParticipantPublicKey,
    ) -> RedisResult<self::impls::SumDictDelete> {
        // https://redis.io/commands/hdel
        // > Return value
        //   Integer reply: the number of fields that were removed from the hash,
        //   not including specified but non existing fields.
        self.connection
            .hdel("sum_dict", PublicSigningKeyWrite::from(pk))
            .await
    }

    // Returns the length of the [`SumDict`].
    pub async fn sum_dict_len(&mut self) -> RedisResult<u64> {
        // https://redis.io/commands/hlen
        // > Return value
        //   Integer reply: number of fields in the hash, or 0 when key does not exist.
        self.connection.hlen("sum_dict").await
    }

    // Returns the [`SumParticipantPublicKey`] of the [`SumDict`] or an empty list when the
    // [`SumDict`] does not exist.
    pub async fn sum_pks(
        &mut self,
    ) -> RedisResult<std::collections::HashSet<SumParticipantPublicKey>> {
        // https://redis.io/commands/hkeys
        // > Return value:
        //   Array reply: list of fields in the hash, or an empty list when key does not exist.
        let result: std::collections::HashSet<PublicSigningKeyRead> =
            self.connection.hkeys("sum_dict").await?;
        let sum_pks = result.into_iter().map(|pk| pk.into()).collect();

        Ok(sum_pks)
    }

    // Removes an update pk from the the `update_participants` set.
    pub async fn remove_update_participant(
        &mut self,
        update_pk: &UpdateParticipantPublicKey,
    ) -> RedisResult<u64> {
        self.connection
            .srem(
                "update_participants",
                PublicSigningKeyWrite::from(update_pk),
            )
            .await
    }

    pub async fn mask_submitted_set(&mut self) -> RedisResult<Vec<SumParticipantPublicKey>> {
        let result: Vec<PublicSigningKeyRead> =
            self.connection.smembers("update_submitted").await?;
        let sum_pks = result.into_iter().map(|pk| pk.into()).collect();
        Ok(sum_pks)
    }

    // Returns all keys in the current database
    pub async fn keys(&mut self) -> RedisResult<Vec<String>> {
        self.connection.keys("*").await
    }

    /// Returns the [`SeedDict`] entry for the given ['SumParticipantPublicKey'] or an empty map
    /// when a [`SeedDict`] entry does not exist.
    pub async fn seed_dict_for_sum_pk(
        &mut self,
        sum_pk: &SumParticipantPublicKey,
    ) -> RedisResult<HashMap<UpdateParticipantPublicKey, modalic_core::mask::EncryptedMaskSeed>>
    {
        debug!(
            "get seed dictionary for sum participant with pk {:?}",
            sum_pk
        );
        // https://redis.io/commands/hgetall
        // > Return value
        //   Array reply: list of fields and their values stored in the hash, or an empty
        //   list when key does not exist.
        let result: Vec<(PublicSigningKeyRead, EncryptedMaskSeedRead)> = self
            .connection
            .hgetall(PublicSigningKeyWrite::from(sum_pk))
            .await?;
        let seed_dict = result
            .into_iter()
            .map(|(pk, seed)| (pk.into(), seed.into()))
            .collect();

        Ok(seed_dict)
    }

    /// Deletes all data in the current database.
    pub async fn flush_db(&mut self) -> RedisResult<()> {
        debug!("flush current database");
        // https://redis.io/commands/flushdb
        // > This command never fails.
        redis::cmd("FLUSHDB")
            .arg("ASYNC")
            .query_async(&mut self.connection)
            .await
    }
}
