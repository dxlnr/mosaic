#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(
    doc,
    forbid(rustdoc::broken_intra_doc_links, rustdoc::private_intra_doc_links)
)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/xaynetwork/xaynet/master/assets/xaynet_banner.png",
    html_favicon_url = "https://raw.githubusercontent.com/xaynetwork/xaynet/master/assets/favicon.png",
    issue_tracker_base_url = "https://github.com/xaynetwork/xaynet/issues"
)]
//! `xaynet_core` provides basic building blocks for implementing the
//! _Privacy-Enhancing Technology_ (PET), a privacy preserving
//! protocol for federated machine learning. Download the [whitepaper]
//! for an introduction.
//!
//! [whitepaper]: https://uploads-ssl.webflow.com/5f0c5c0bb18a279f0a62919e/5f157004da6585f299fa542b_XayNet%20Whitepaper%202.1.pdf

pub mod common;
pub mod crypto;
pub mod mask;
pub mod message;
pub mod model;
#[cfg(any(feature = "testutils", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "testutils")))]
pub mod testutils;

use std::collections::HashMap;

use thiserror::Error;

use self::crypto::{
    encrypt::{PublicEncryptKey, SecretEncryptKey},
    sign::{PublicSigningKey, SecretSigningKey, Signature},
};

#[derive(Error, Debug)]
#[error("initialization failed: insufficient system entropy to generate secrets")]
/// An error related to insufficient system entropy for secrets at program startup.
pub struct InitError;

/// A public encryption key that identifies a coordinator.
pub type CoordinatorPublicKey = PublicEncryptKey;

/// A secret encryption key that belongs to the public key of a
/// coordinator.
pub type CoordinatorSecretKey = SecretEncryptKey;

/// A public signature key that identifies a participant.
pub type ParticipantPublicKey = PublicSigningKey;

/// A secret signature key that belongs to the public key of a
/// participant.
pub type ParticipantSecretKey = SecretSigningKey;

/// A public signature key that identifies a sum participant.
pub type SumParticipantPublicKey = ParticipantPublicKey;

/// A secret signature key that belongs to the public key of a sum
/// participant.
pub type SumParticipantSecretKey = ParticipantSecretKey;

/// A public encryption key generated by a sum participant. It is used
/// by the update participants to encrypt their masking seed for each
/// sum participant.
pub type SumParticipantEphemeralPublicKey = PublicEncryptKey;

/// The secret counterpart of [`SumParticipantEphemeralPublicKey`]
pub type SumParticipantEphemeralSecretKey = SecretEncryptKey;

/// A public signature key that identifies an update participant.
pub type UpdateParticipantPublicKey = ParticipantPublicKey;

/// A secret signature key that belongs to the public key of an update
/// participant.
pub type UpdateParticipantSecretKey = ParticipantSecretKey;

/// A signature to prove a participant's eligibility for a task.
pub type ParticipantTaskSignature = Signature;

/// A dictionary created during the sum phase of the protocol. It maps the public key of every sum
/// participant to the ephemeral public key generated by that sum participant.
pub type SumDict = HashMap<SumParticipantPublicKey, SumParticipantEphemeralPublicKey>;

/// Local seed dictionaries are sent by update participants. They contain the participant's masking
/// seed, encrypted with the ephemeral public key of each sum participant.
pub type LocalSeedDict = HashMap<SumParticipantPublicKey, mask::seed::EncryptedMaskSeed>;

/// A dictionary created during the update phase of the protocol. The global seed dictionary is
/// built from the local seed dictionaries sent by the update participants. It maps each sum
/// participant to the encrypted masking seeds of all the update participants.
pub type SeedDict = HashMap<SumParticipantPublicKey, UpdateSeedDict>;

/// Values of [`SeedDict`]. Sent to sum participants.
pub type UpdateSeedDict = HashMap<UpdateParticipantPublicKey, mask::seed::EncryptedMaskSeed>;
