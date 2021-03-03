//
// Copyright 2020 Signal Messenger, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//

/// A dummy StoreContext usable with InMemorySignalProtocolStore.
public struct NullContext: StoreContext {
    public init() {}
}

public class InMemorySignalProtocolStore: IdentityKeyStore, PreKeyStore, SignedPreKeyStore, SessionStore, SenderKeyStore {
    private var publicKeys: [ProtocolAddress: IdentityKey] = [:]
    private var privateKey: IdentityKeyPair
    private var deviceId: UInt32
    private var prekeyMap: [UInt32: PreKeyRecord] = [:]
    private var signedPrekeyMap: [UInt32: SignedPreKeyRecord] = [:]
    private var sessionMap: [ProtocolAddress: SessionRecord] = [:]
    private var senderKeyMap: [SenderKeyName: SenderKeyRecord] = [:]

    public init() {
        privateKey = IdentityKeyPair.generate()
        deviceId = UInt32.random(in: 0...65535)
    }

    public init(identity: IdentityKeyPair, deviceId: UInt32) {
        self.privateKey = identity
        self.deviceId = deviceId
    }

    public func identityKeyPair(context: StoreContext) throws -> IdentityKeyPair {
        return privateKey
    }

    public func localRegistrationId(context: StoreContext) throws -> UInt32 {
        return deviceId
    }

    public func saveIdentity(_ identity: IdentityKey, for address: ProtocolAddress, context: StoreContext) throws -> Bool {
        if publicKeys.updateValue(identity, forKey: address) == nil {
            return false; // newly created
        } else {
            return true
        }
    }

    public func isTrustedIdentity(_ identity: IdentityKey, for address: ProtocolAddress, direction: Direction, context: StoreContext) throws -> Bool {
        if let pk = publicKeys[address] {
            return pk == identity
        } else {
            return true // tofu
        }
    }

    public func identity(for address: ProtocolAddress, context: StoreContext) throws -> IdentityKey? {
        return publicKeys[address]
    }

    public func loadPreKey(id: UInt32, context: StoreContext) throws -> PreKeyRecord {
        if let record = prekeyMap[id] {
            return record
        } else {
            throw SignalError.invalidKeyIdentifier("no prekey with this identifier")
        }
    }

    public func storePreKey(_ record: PreKeyRecord, id: UInt32, context: StoreContext) throws {
        prekeyMap[id] = record
    }

    public func removePreKey(id: UInt32, context: StoreContext) throws {
        prekeyMap.removeValue(forKey: id)
    }

    public func loadSignedPreKey(id: UInt32, context: StoreContext) throws -> SignedPreKeyRecord {
        if let record = signedPrekeyMap[id] {
            return record
        } else {
            throw SignalError.invalidKeyIdentifier("no signed prekey with this identifier")
        }
    }

    public func storeSignedPreKey(_ record: SignedPreKeyRecord, id: UInt32, context: StoreContext) throws {
        signedPrekeyMap[id] = record
    }

    public func loadSession(for address: ProtocolAddress, context: StoreContext) throws -> SessionRecord? {
        return sessionMap[address]
    }

    public func storeSession(_ record: SessionRecord, for address: ProtocolAddress, context: StoreContext) throws {
        sessionMap[address] = record
    }

    public func storeSenderKey(name: SenderKeyName, record: SenderKeyRecord, context: StoreContext) throws {
        senderKeyMap[name] = record
    }

    public func loadSenderKey(name: SenderKeyName, context: StoreContext) throws -> SenderKeyRecord? {
        return senderKeyMap[name]
    }
}
