//
// Copyright 2020 Signal Messenger, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//

import SignalFfi
import Foundation

public class ServerCertificate: ClonableHandleOwner {
    public init<Bytes: ContiguousBytes>(_ bytes: Bytes) throws {
        let handle: OpaquePointer? = try bytes.withUnsafeBytes {
            var result: OpaquePointer?
            try checkError(signal_server_certificate_deserialize(&result, $0.baseAddress?.assumingMemoryBound(to: UInt8.self), $0.count))
            return result
        }
        super.init(owned: handle!)
    }

    // For testing
    public init(keyId: UInt32, publicKey: PublicKey, trustRoot: PrivateKey) throws {
        var result: OpaquePointer?
        try checkError(signal_server_certificate_new(&result, keyId, publicKey.nativeHandle, trustRoot.nativeHandle))
        super.init(owned: result!)
    }

    internal override init(owned handle: OpaquePointer) {
        super.init(owned: handle)
    }

    internal override init(borrowing handle: OpaquePointer?) {
        super.init(borrowing: handle)
    }

    internal override class func destroyNativeHandle(_ handle: OpaquePointer) -> SignalFfiErrorRef? {
        return signal_server_certificate_destroy(handle)
    }

    public var keyId: UInt32 {
        return failOnError {
            try invokeFnReturningInteger {
                signal_server_certificate_get_key_id($0, nativeHandle)
            }
        }
    }

    public func serialize() -> [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_server_certificate_get_serialized($0, $1, nativeHandle)
            }
        }
    }

    public var certificateBytes: [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_server_certificate_get_certificate($0, $1, nativeHandle)
            }
        }
    }

    public var signatureBytes: [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_server_certificate_get_signature($0, $1, nativeHandle)
            }
        }
    }

    public var publicKey: PublicKey {
        return failOnError {
            try invokeFnReturningPublicKey {
                signal_server_certificate_get_key($0, nativeHandle)
            }
        }
    }
}

public class SenderCertificate: ClonableHandleOwner {
    public init<Bytes: ContiguousBytes>(_ bytes: Bytes) throws {
        let handle: OpaquePointer? = try bytes.withUnsafeBytes {
            var result: OpaquePointer?
            try checkError(signal_sender_certificate_deserialize(&result, $0.baseAddress?.assumingMemoryBound(to: UInt8.self), $0.count))
            return result
        }
        super.init(owned: handle!)
    }

    // For testing
    public init(sender: SealedSenderAddress, publicKey: PublicKey, expiration: UInt64, signerCertificate: ServerCertificate, signerKey: PrivateKey) throws {
        var result: OpaquePointer?
        try checkError(signal_sender_certificate_new(&result,
                                                     sender.uuidString,
                                                     sender.e164,
                                                     sender.deviceId,
                                                     publicKey.nativeHandle,
                                                     expiration,
                                                     signerCertificate.nativeHandle,
                                                     signerKey.nativeHandle))
        super.init(owned: result!)
    }

    internal override init(owned handle: OpaquePointer) {
        super.init(owned: handle)
    }

    internal override class func destroyNativeHandle(_ handle: OpaquePointer) -> SignalFfiErrorRef? {
        return signal_sender_certificate_destroy(handle)
    }

    public var expiration: UInt64 {
        return failOnError {
            try invokeFnReturningInteger {
                signal_sender_certificate_get_expiration($0, nativeHandle)
            }
        }
    }

    public var deviceId: UInt32 {
        return failOnError {
            try invokeFnReturningInteger {
                signal_sender_certificate_get_device_id($0, nativeHandle)
            }
        }
    }

    public func serialize() -> [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_sender_certificate_get_serialized($0, $1, nativeHandle)
            }
        }
    }

    public var certificateBytes: [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_sender_certificate_get_certificate($0, $1, nativeHandle)
            }
        }
    }

    public var signatureBytes: [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_sender_certificate_get_signature($0, $1, nativeHandle)
            }
        }
    }

    public var publicKey: PublicKey {
        return failOnError {
            try invokeFnReturningPublicKey {
                signal_sender_certificate_get_key($0, nativeHandle)
            }
        }
    }

    public var senderUuid: String {
        return failOnError {
            try invokeFnReturningString {
                signal_sender_certificate_get_sender_uuid($0, nativeHandle)
            }
        }
    }

    public var senderE164: String? {
        return failOnError {
            try invokeFnReturningOptionalString {
                signal_sender_certificate_get_sender_e164($0, nativeHandle)
            }
        }
    }

    public var sender: SealedSenderAddress {
        return try! SealedSenderAddress(e164: self.senderE164, uuidString: self.senderUuid, deviceId: self.deviceId)
    }

    public var serverCertificate: ServerCertificate {
        var handle: OpaquePointer?
        failOnError(signal_sender_certificate_get_server_certificate(&handle, nativeHandle))
        return ServerCertificate(owned: handle!)
    }

    public func validate(trustRoot: PublicKey, time: UInt64) throws -> Bool {
        var result: Bool = false
        try checkError(signal_sender_certificate_validate(&result, nativeHandle, trustRoot.nativeHandle, time))
        return result
    }
}

public func sealedSenderEncrypt<Bytes: ContiguousBytes>(message: Bytes,
                                                        for address: ProtocolAddress,
                                                        from senderCert: SenderCertificate,
                                                        sessionStore: SessionStore,
                                                        identityStore: IdentityKeyStore,
                                                        context: StoreContext) throws -> [UInt8] {
    return try message.withUnsafeBytes { messageBytes in
        try context.withOpaquePointer { context in
            try withSessionStore(sessionStore) { ffiSessionStore in
                try withIdentityKeyStore(identityStore) { ffiIdentityStore in
                    try invokeFnReturningArray {
                        signal_sealed_session_cipher_encrypt($0, $1,
                                                             address.nativeHandle, senderCert.nativeHandle,
                                                             messageBytes.baseAddress?.assumingMemoryBound(to: UInt8.self),
                                                             messageBytes.count,
                                                             ffiSessionStore, ffiIdentityStore, context)
                    }
                }
            }
        }
    }
}

public class UnidentifiedSenderMessageContent: ClonableHandleOwner {
    public init<Bytes: ContiguousBytes>(message: Bytes,
                                        identityStore: IdentityKeyStore,
                                        context: StoreContext) throws {
        var result: OpaquePointer?
        try message.withUnsafeBytes { messageBytes in
            try context.withOpaquePointer { context in
                try withIdentityKeyStore(identityStore) { ffiIdentityStore in
                    try checkError(
                        signal_sealed_session_cipher_decrypt_to_usmc(
                            &result,
                            messageBytes.baseAddress?.assumingMemoryBound(to: UInt8.self),
                            messageBytes.count,
                            ffiIdentityStore,
                            context))
                }
            }
        }
        super.init(owned: result!)
    }

    internal override class func destroyNativeHandle(_ handle: OpaquePointer) -> SignalFfiErrorRef? {
        return signal_unidentified_sender_message_content_destroy(handle)
    }

    public var senderCertificate: SenderCertificate {
        var result: OpaquePointer?
        failOnError(signal_unidentified_sender_message_content_get_sender_cert(&result, self.nativeHandle))
        return SenderCertificate(owned: result!)
    }

    public var messageType: CiphertextMessage.MessageType {
        let rawType = failOnError {
            try invokeFnReturningInteger {
                signal_unidentified_sender_message_content_get_msg_type($0, self.nativeHandle)
            }
        }
        return .init(rawValue: rawType)
    }

    public var contents: [UInt8] {
        return failOnError {
            try invokeFnReturningArray {
                signal_unidentified_sender_message_content_get_contents($0, $1, self.nativeHandle)
            }
        }
    }
}

public struct SealedSenderAddress: Hashable {
    public var e164: String?
    public var uuidString: String
    public var deviceId: UInt32

    public init(e164: String?, uuidString: String, deviceId: UInt32) throws {
        self.e164 = e164
        self.uuidString = uuidString
        self.deviceId = deviceId
    }
}

public struct SealedSenderResult {
    public var message: [UInt8]
    public var sender: SealedSenderAddress
}

public func sealedSenderDecrypt<Bytes: ContiguousBytes>(message: Bytes,
                                                        from localAddress: SealedSenderAddress,
                                                        trustRoot: PublicKey,
                                                        timestamp: UInt64,
                                                        sessionStore: SessionStore,
                                                        identityStore: IdentityKeyStore,
                                                        preKeyStore: PreKeyStore,
                                                        signedPreKeyStore: SignedPreKeyStore,
                                                        context: StoreContext) throws -> SealedSenderResult {
    var senderE164: UnsafePointer<CChar>?
    var senderUUID: UnsafePointer<CChar>?
    var senderDeviceId: UInt32 = 0

    let plaintext = try message.withUnsafeBytes { messageBytes in
        try context.withOpaquePointer { context in
            try withSessionStore(sessionStore) { ffiSessionStore in
                try withIdentityKeyStore(identityStore) { ffiIdentityStore in
                    try withPreKeyStore(preKeyStore) { ffiPreKeyStore in
                        try withSignedPreKeyStore(signedPreKeyStore) { ffiSignedPreKeyStore in
                            try invokeFnReturningArray {
                                signal_sealed_session_cipher_decrypt(
                                    $0,
                                    $1,
                                    &senderE164,
                                    &senderUUID,
                                    &senderDeviceId,
                                    messageBytes.baseAddress?.assumingMemoryBound(to: UInt8.self),
                                    messageBytes.count,
                                    trustRoot.nativeHandle,
                                    timestamp,
                                    localAddress.e164,
                                    localAddress.uuidString,
                                    localAddress.deviceId,
                                    ffiSessionStore,
                                    ffiIdentityStore,
                                    ffiPreKeyStore,
                                    ffiSignedPreKeyStore,
                                    context)
                            }
                        }
                    }
                }
            }
        }
    }

    defer {
        signal_free_string(senderE164)
        signal_free_string(senderUUID)
    }

    return SealedSenderResult(message: plaintext,
                              sender: try SealedSenderAddress(e164: senderE164.map(String.init(cString:)),
                                                              uuidString: String(cString: senderUUID!),
                                                              deviceId: senderDeviceId))
}
