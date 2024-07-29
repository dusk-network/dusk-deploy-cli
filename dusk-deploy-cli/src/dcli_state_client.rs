// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::rusk_http_client::RuskHttpClient;
use crate::Error;
use execution_core::{BlsScalar, Note, PublicKey, ViewKey};
use purse::StateClient;
use std::sync::Mutex;
use wallet::StateClient;

pub struct DCliStateClient {
    pub inner: Mutex<InnerState>,
}

struct InnerState {
    pub client: RuskHttpClient,
    // cache: Arc<Cache>,
}

pub type BlockHeight = u64;

pub type EnrichedNote = (Note, BlockHeight);

impl DCliStateClient {
    pub fn new(rusk_http_client: RuskHttpClient) -> Self {
        let inner = InnerState {
            client: rusk_http_client,
        };
        Self {
            inner: Mutex::new(inner),
        }
    }
}

impl StateClient for DCliStateClient {
    /// Error returned by the node client.
    type Error = Error;

    /// Find notes for a view key, starting from the given block height.
    fn fetch_notes(&self, vk: &ViewKey) -> Result<Vec<EnrichedNote>, Self::Error> {
        let psk = vk.public_spend_key();
        let state = self.inner.lock().unwrap();

        Ok(state
            .cache
            .notes(&psk)?
            .into_iter()
            .map(|data| (data.note, data.height))
            .collect())
    }

    /// Fetch the current anchor of the state.
    fn fetch_anchor(&self) -> Result<BlsScalar, Self::Error> {
        let state = self.inner.lock().unwrap();

        self.status("Fetching anchor...");

        let anchor = state
            .client
            .contract_query::<(), 0>(TRANSFER_CONTRACT, "root", &())
            .wait()?;
        self.status("Anchor received!");
        let anchor = rkyv::from_bytes(&anchor).map_err(|_| Error::Rkyv)?;
        Ok(anchor)
    }

    /// Asks the node to return the nullifiers that already exist from the given
    /// nullifiers.
    fn fetch_existing_nullifiers(
        &self,
        _nullifiers: &[BlsScalar],
    ) -> Result<Vec<BlsScalar>, Self::Error> {
        Ok(vec![])
    }

    /// Queries the node to find the opening for a specific note.
    fn fetch_opening(
        &self,
        note: &Note,
    ) -> Result<PoseidonOpening<(), POSEIDON_TREE_DEPTH, 4>, Self::Error> {
        let state = self.inner.lock().unwrap();

        self.status("Fetching opening notes...");

        let data = state
            .client
            .contract_query::<_, 1024>(TRANSFER_CONTRACT, "opening", note.pos())
            .wait()?;

        self.status("Opening notes received!");

        let branch = rkyv::from_bytes(&data).map_err(|_| Error::Rkyv)?;
        Ok(branch)
    }
}
