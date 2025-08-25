// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Error;
use bip39::{Language, Mnemonic, Seed};

pub struct SeedUtil;

impl SeedUtil {
    /// converts seed phrase into a binary seed
    pub fn seed_from_phrase(phrase: impl AsRef<str>) -> Result<[u8; 64], Error> {
        let mnemonic = Mnemonic::from_phrase(phrase.as_ref(), Language::English)
            .map_err(|_| Error::InvalidMnemonicPhrase)?;
        let seed_obj = Seed::new(&mnemonic, "");
        let mut seed = [0u8; 64];
        seed.copy_from_slice(seed_obj.as_bytes());
        Ok(seed)
    }
}
