//
// Copyright 2020 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

fn main() {
    let protos = [
        "src/proto/fingerprint.proto",
        "src/proto/storage.proto",
        "src/proto/sealed_sender.proto",
        "src/proto/wire.proto",
    ];
    prost_build::compile_protos(&protos, &["src"]).unwrap();
    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto);
    }
}
