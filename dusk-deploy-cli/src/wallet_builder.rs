// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_state_client::DCliStateClient;
use crate::rusk_http_client::RuskHttpClient;
use crate::Error;
use execution_core::transfer::Transaction;
use rusk_prover::{LocalProver, Prover, UnprovenTransaction};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use wallet::{ProverClient, StateClient, Store, Wallet};

#[derive(Debug, Clone)]
pub struct DcliStore;

impl Store for DcliStore {
    type Error = ();

    fn get_seed(&self) -> Result<[u8; 64], Self::Error> {
        Ok([0; 64])
    }
}

#[derive(Default)]
pub struct DcliProverClient {
    pub prover: LocalProver,
}

impl Debug for DcliProverClient {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl wallet::ProverClient for DcliProverClient {
    type Error = Error;
    /// Requests that a node prove the given transaction and later propagates it
    fn compute_proof_and_propagate(
        &self,
        utx: &UnprovenTransaction,
    ) -> Result<Transaction, Self::Error> {
        let utx_bytes = &utx.to_var_bytes()[..];
        let proof = self.prover.prove_execute(utx_bytes)?;
        let proof = Proof::from_slice(&proof).map_err(Error::Serialization)?;
        let tx = utx.clone().gen_transaction(proof);

        //Propagate is not required yet

        Ok(tx.into())
    }
}

pub struct WalletBuilder<S, SC, PC>;

impl<S, SC, PC> WalletBuilder<S, SC, PC>
where
    S: Store,
    SC: StateClient,
    PC: ProverClient,
{
    pub fn build(url: impl AsRef<str>) -> Result<Wallet<S, SC, PC>, Error> {
        // let cache = Arc::new(RwLock::new(HashMap::new()));

        let wallet = wallet::Wallet::new(
            DcliStore,
            DCliStateClient::new(RuskHttpClient::new(url.to_string())),
            DcliProverClient::default(),
        );
        Ok(wallet)
    }
}
