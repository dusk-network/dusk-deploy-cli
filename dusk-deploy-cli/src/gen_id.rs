// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::Params;
use execution_core::CONTRACT_ID_BYTES;

/// Generate a [`ContractId`] address from:
/// - slice of bytes,
/// - nonce
/// - owner
pub fn gen_contract_id(bytes: impl AsRef<[u8]>, nonce: u64, owner: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Params::new().hash_length(CONTRACT_ID_BYTES).to_state();
    hasher.update(bytes.as_ref());
    hasher.update(&nonce.to_le_bytes()[..]);
    hasher.update(owner.as_ref());
    hasher
        .finalize()
        .as_bytes()
        .try_into()
        .expect("the hash result is exactly `CONTRACT_ID_BYTES` long")
}
