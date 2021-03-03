//
// Copyright 2020 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

#![deny(unsafe_code)]

mod address;
mod consts;
mod crypto;
mod curve;
pub mod error;
mod fingerprint;
mod group_cipher;
mod identity_key;
mod kdf;
mod proto;
mod protocol;
mod ratchet;
mod sealed_sender;
mod sender_keys;
mod session;
mod session_cipher;
mod state;
mod storage;
mod utils;

use error::Result;

pub use {
    address::ProtocolAddress,
    curve::{KeyPair, PrivateKey, PublicKey},
    error::SignalProtocolError,
    fingerprint::{DisplayableFingerprint, Fingerprint, ScannableFingerprint},
    group_cipher::{
        create_sender_key_distribution_message, group_decrypt, group_encrypt,
        process_sender_key_distribution_message,
    },
    identity_key::{IdentityKey, IdentityKeyPair},
    kdf::HKDF,
    protocol::{
        CiphertextMessage, CiphertextMessageType, PreKeySignalMessage,
        SenderKeyDistributionMessage, SenderKeyMessage, SignalMessage,
    },
    ratchet::{
        initialize_alice_session_record, initialize_bob_session_record,
        AliceSignalProtocolParameters, BobSignalProtocolParameters,
    },
    sealed_sender::{
        sealed_sender_decrypt, sealed_sender_decrypt_to_usmc, sealed_sender_encrypt,
        SealedSenderDecryptionResult, SenderCertificate, ServerCertificate,
        UnidentifiedSenderMessage, UnidentifiedSenderMessageContent,
    },
    sender_keys::{SenderKeyName, SenderKeyRecord},
    session::{process_prekey, process_prekey_bundle},
    session_cipher::{
        message_decrypt, message_decrypt_prekey, message_decrypt_signal, message_encrypt,
    },
    state::{PreKeyBundle, PreKeyRecord, SessionRecord, SignedPreKeyRecord},
    storage::{
        Context, Direction, IdentityKeyStore, InMemIdentityKeyStore, InMemPreKeyStore,
        InMemSenderKeyStore, InMemSessionStore, InMemSignalProtocolStore, InMemSignedPreKeyStore,
        PreKeyStore, ProtocolStore, SenderKeyStore, SessionStore, SignedPreKeyStore,
    },
};
