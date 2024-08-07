// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Error;
use wallet::Store;

#[derive(Debug, Clone)]
pub struct DCliStore {
    pub seed: [u8; 64],
}

impl DCliStore {
    pub fn new(seed: &[u8; 64]) -> Self {
        Self { seed: seed.clone() }
    }
}

impl Store for DCliStore {
    type Error = Error;

    fn get_seed(&self) -> Result<[u8; 64], Self::Error> {
        Ok(self.seed.clone())
    }
}
