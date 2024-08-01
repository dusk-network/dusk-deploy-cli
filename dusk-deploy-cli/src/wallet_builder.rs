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
        let seed = hex::decode("7965013909185294fa0f0d2a2be850ee89389e45d17e0c7da9a7588901648086c5b3ac52d95b6fd421104b6a77ca21772f0a041f031c3c8039ae3b24c48467bd")
            .expect("decoding seed should succeed");
        assert_eq!(seed.len(), 64);
        let a: [u8; 64] = seed
            .try_into()
            .expect("seed conversion to array should succeed");
        println!("seed={}", hex::encode(a.clone()));
        Ok(a)
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
