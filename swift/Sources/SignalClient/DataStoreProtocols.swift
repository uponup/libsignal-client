//
// Copyright 2020 Signal Messenger, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//

import SignalFfi

public enum Direction {
    case sending
    case receiving
}

/// A marker protocol, which must be downcast to use in any particular store.
///
/// Essentially `Any`, but still able to catch typos when calling something that uses stores.
public protocol StoreContext {}

public protocol IdentityKeyStore: AnyObject {
    func identityKeyPair(context: StoreContext) throws -> IdentityKeyPair
    func localRegistrationId(context: StoreContext) throws -> UInt32
    func saveIdentity(_ identity: IdentityKey, for address: ProtocolAddress, context: StoreContext) throws -> Bool
    func isTrustedIdentity(_ identity: IdentityKey, for address: ProtocolAddress, direction: Direction, context: StoreContext) throws -> Bool
    func identity(for address: ProtocolAddress, context: StoreContext) throws -> IdentityKey?
}

public protocol PreKeyStore: AnyObject {
    func loadPreKey(id: UInt32, context: StoreContext) throws -> PreKeyRecord
    func storePreKey(_ record: PreKeyRecord, id: UInt32, context: StoreContext) throws
    func removePreKey(id: UInt32, context: StoreContext) throws
}

public protocol SignedPreKeyStore: AnyObject {
    func loadSignedPreKey(id: UInt32, context: StoreContext) throws -> SignedPreKeyRecord
    func storeSignedPreKey(_ record: SignedPreKeyRecord, id: UInt32, context: StoreContext) throws
}

public protocol SessionStore: AnyObject {
    func loadSession(for address: ProtocolAddress, context: StoreContext) throws -> SessionRecord?
    func storeSession(_ record: SessionRecord, for address: ProtocolAddress, context: StoreContext) throws
}

public protocol SenderKeyStore: AnyObject {
    func storeSenderKey(name: SenderKeyName, record: SenderKeyRecord, context: StoreContext) throws
    func loadSenderKey(name: SenderKeyName, context: StoreContext) throws -> SenderKeyRecord?
}
