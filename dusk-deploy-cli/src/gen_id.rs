// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::{Params, State};

/// Generate a [`ContractId`] address from:
/// - slice of bytes,
/// - nonce
/// - owner
pub fn gen_contract_id(bytes: impl AsRef<[u8]>, nonce: u64, owner: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(bytes.as_ref());
    hasher.update(nonce.to_le_bytes());
    hasher.update(owner.as_ref());
    let hash_bytes = hasher.finalize();
    hash_bytes
}

/// Hashes scalars and arbitrary slices of bytes using Blake2b, returning an
/// array of 32 bytes.
///
/// This hash cannot be proven inside a circuit, if that is desired, use
/// `poseidon_hash` instead.
pub struct Hasher {
    state: State,
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher {
            state: Params::new().hash_length(64).to_state(),
        }
    }
}

impl Hasher {
    /// Create new hasher instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process data, updating the internal state.
    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        self.state.update(data.as_ref());
    }

    /// Retrieve result and consume hasher instance.
    pub fn finalize(self) -> [u8; 32] {
        let hash = self.state.finalize();
        let mut a = [0u8; 32];
        a.clone_from_slice(&hash.as_array()[..32]);
        a
    }
}
