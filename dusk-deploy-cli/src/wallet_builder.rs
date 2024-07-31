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
pub struct DCliStore;

impl Store for DCliStore {
    type Error = ();

    fn get_seed(&self) -> Result<[u8; 64], Self::Error> {
        Ok([0; 64])
    }
}

pub struct WalletBuilder;

impl WalletBuilder {
    pub fn build(
        url_state: impl AsRef<str>,
        url_prover: impl AsRef<str>,
    ) -> Result<Wallet<DCliStore, DCliStateClient, DCliProverClient>, Error> {
        let state_client = RuskHttpClient::new(url_state.as_ref().to_string());
        let prover_client = RuskHttpClient::new(url_prover.as_ref().to_string());

        let wallet = wallet::Wallet::new(
            DCliStore,
            DCliStateClient::new(state_client.clone()),
            DCliProverClient::new(state_client.clone(), prover_client.clone()),
        );
        Ok(wallet)
    }
}
