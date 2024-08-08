// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::cmp::max;
use crate::block::Block;
use crate::Error;
use dusk_bytes::Serializable;
use execution_core::transfer::{AccountData, ContractId, TreeLeaf};
use execution_core::{BlsPublicKey, BlsScalar, Note, ViewKey};
use poseidon_merkle::Opening as PoseidonOpening;
use rusk_http_client::RuskHttpClient;
use rusk_http_client::{ContractInquirer, StreamAux};
use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, RwLock};
use tracing::info;
use wallet::StateClient;

const CONTRACT_ID_BYTES: usize = 32;

#[inline]
const fn reserved(b: u8) -> ContractId {
    let mut bytes = [0u8; CONTRACT_ID_BYTES];
    bytes[0] = b;
    bytes
}

const TRANSFER_CONTRACT: ContractId = reserved(0x1);

const POSEIDON_TREE_DEPTH: usize = 17;

const TRANSFER_CONTRACT_STR: &str =
    "0100000000000000000000000000000000000000000000000000000000000000";

const ITEM_LEN: usize = mem::size_of::<TreeLeaf>();

pub struct DCliStateClient {
    pub client: RuskHttpClient,
    pub cache: Arc<RwLock<HashMap<Vec<u8>, DummyCacheItem>>>,
    pub start_block_height: u64,
}

#[derive(Default, Debug, Clone)]
pub struct DummyCacheItem {
    notes: Vec<(Note, u64)>,
    last_height: u64,
}

impl DummyCacheItem {
    fn add(&mut self, note: Note, block_height: u64) {
        if !self.notes.contains(&(note.clone(), block_height)) {
            self.notes.push((note.clone(), block_height));
            self.last_height = block_height;
        }
    }
}

pub type BlockHeight = u64;

pub type EnrichedNote = (Note, BlockHeight);

impl DCliStateClient {
    pub fn new(rusk_http_client: RuskHttpClient, start_block_height: u64) -> Self {
        let cache = Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
        Self {
            client: rusk_http_client,
            cache,
            start_block_height,
        }
    }
}

impl Debug for DCliStateClient {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl StateClient for DCliStateClient {
    /// Error returned by the node client.
    type Error = Error;

    /// Find notes for a view key, starting from the given block height.
    fn fetch_notes(&self, vk: &ViewKey) -> Result<Vec<EnrichedNote>, Error> {
        let cache_read = self.cache.read().unwrap();
        let mut vk_cache = if cache_read.contains_key(&vk.to_bytes().to_vec()) {
            cache_read.get(&vk.to_bytes().to_vec()).unwrap().clone()
        } else {
            DummyCacheItem::default()
        };

        let start_height = max(self.start_block_height, vk_cache.last_height);
        info!("Requesting notes from height {}", start_height);
        let mut response_notes = Vec::new();
        let mut stream = ContractInquirer::query_contract_with_feeder(
            &self.client,
            start_height,
            TRANSFER_CONTRACT,
            "leaves_from_height",
        )
        .wait()?;
        StreamAux::find_items::<TreeLeaf, ITEM_LEN>(
            |leaf| {
                if vk.owns(leaf.note.stealth_address()) {
                    response_notes.push((leaf.block_height, leaf.note.clone()))
                }
            },
            &mut stream,
        )?;

        for (block_height, note) in response_notes {
            // Filter out duplicated notes and update the last
            vk_cache.add(note, block_height)
        }
        drop(cache_read);
        self.cache
            .write()
            .unwrap()
            .insert(vk.to_bytes().to_vec(), vk_cache.clone());

        Ok(vk_cache.notes)
    }

    /// Fetch the current anchor of the state.
    fn fetch_anchor(&self) -> Result<BlsScalar, Self::Error> {
        let anchor = self
            .client
            .contract_query::<(), 0>(TRANSFER_CONTRACT_STR, "root", &())
            .wait()?;
        let anchor = rkyv::from_bytes(&anchor).map_err(|_| Error::Rkyv)?;
        Ok(anchor)
    }

    /// Asks the node to return the nullifiers that already exist from the given
    /// nullifiers.
    fn fetch_existing_nullifiers(
        &self,
        nullifiers: &[BlsScalar],
    ) -> Result<Vec<BlsScalar>, Self::Error> {
        if nullifiers.is_empty() {
            return Ok(vec![]);
        }
        let nullifiers = nullifiers.to_vec();
        let data = self
            .client
            .contract_query::<_, 1024>(TRANSFER_CONTRACT_STR, "existing_nullifiers", &nullifiers)
            .wait()?;

        let nullifiers = rkyv::from_bytes(&data).map_err(|_| Error::Rkyv)?;

        Ok(nullifiers)
    }

    /// Queries the node to find the opening for a specific note.
    fn fetch_opening(
        &self,
        note: &Note,
    ) -> Result<PoseidonOpening<(), POSEIDON_TREE_DEPTH>, Self::Error> {
        let data = self
            .client
            .contract_query::<_, 1024>(TRANSFER_CONTRACT_STR, "opening", note.pos())
            .wait()?;

        let branch = rkyv::from_bytes(&data).map_err(|_| Error::Rkyv)?;
        Ok(branch)
    }

    fn fetch_account(&self, pk: &BlsPublicKey) -> Result<AccountData, Self::Error> {
        let data = self
            .client
            .contract_query::<_, 1024>(TRANSFER_CONTRACT_STR, "account", pk)
            .wait()?;

        let account = rkyv::from_bytes(&data).map_err(|_| Error::Rkyv)?;
        Ok(account)
    }
}
