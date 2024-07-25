// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::Error;
use dusk_wallet::RuskHttpClient;
use std::fmt::Debug;
use wallet::{Store, Wallet};

#[derive(Debug, Clone)]
pub struct DcliStore;

impl Store for DcliStore {
    type Error = ();

    fn get_seed(&self) -> Result<[u8; 64], Self::Error> {
        Ok([0; 64])
    }
}

pub struct WalletBuilder;

impl WalletBuilder {
    pub fn build(
        url: impl AsRef<str>,
    ) -> Result<Wallet<DcliStore, DCliStateClient, DCliProverClient>, Error> {
        // let cache = Arc::new(RwLock::new(HashMap::new()));

        let wallet = wallet::Wallet::new(
            DcliStore,
            DCliStateClient::new(RuskHttpClient::new(url.as_ref().to_string())),
            DCliProverClient::default(),
        );
        Ok(wallet)
    }
}
