//
// Copyright 2020 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use curve25519_dalek::scalar::Scalar;

// Because Rust can't create array references from slices (yet)
pub fn scalar_from_slice_wide(bytes: &[u8]) -> Scalar {
    let mut scalar_bytes = [0u8; 64];
    scalar_bytes.copy_from_slice(bytes);
    Scalar::from_bytes_mod_order_wide(&scalar_bytes)
}

// Because Rust can't create array references from slices (yet)
pub fn scalar_from_slice_canonical(bytes: &[u8]) -> Option<Scalar> {
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(bytes);
    Scalar::from_canonical_bytes(scalar_bytes)
}
