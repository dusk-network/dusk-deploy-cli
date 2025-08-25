// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Error;
use dusk_bytes::Serializable;
use dusk_core::signatures::bls::PublicKey as AccountPublicKey;

pub struct Bs58Util;

impl Bs58Util {
    #[allow(dead_code)]
    /// converts base 58 string into a public key
    pub fn to_public_key(s: impl AsRef<str>) -> Result<AccountPublicKey, Error> {
        let pk_bytes = bs58::decode(s.as_ref()).into_vec()?;
        let mut pk_a = [0u8; AccountPublicKey::SIZE];
        pk_a.copy_from_slice(pk_bytes.as_slice());
        Ok(AccountPublicKey::from_bytes(&pk_a)?)
    }

    #[allow(dead_code)]
    /// converts base 58 string into a binary seed
    pub fn to_seed(bs58_str: impl AsRef<str>) -> Result<[u8; 64], Error> {
        let v = bs58::decode(bs58_str.as_ref()).into_vec()?;
        let mut seed = [0u8; 64];
        seed[0..32].copy_from_slice(&v);
        Ok(seed)
    }
}
