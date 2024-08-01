// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::dcli_store::DCliStore;
use crate::Error;
use rusk_http_client::RuskHttpClient;
use wallet::Wallet;

pub struct WalletBuilder;

impl WalletBuilder {
    pub fn build(
        url_state: impl AsRef<str>,
        url_prover: impl AsRef<str>,
        seed: &[u8; 64],
    ) -> Result<Wallet<DCliStore, DCliStateClient, DCliProverClient>, Error> {
        let state_client = RuskHttpClient::new(url_state.as_ref().to_string());
        let prover_client = RuskHttpClient::new(url_prover.as_ref().to_string());

        Ok(wallet::Wallet::new(
            DCliStore::new(seed),
            DCliStateClient::new(state_client.clone()),
            DCliProverClient::new(state_client.clone(), prover_client.clone()),
        ))
    }
}
