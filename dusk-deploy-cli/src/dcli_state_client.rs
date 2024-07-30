// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::block::Block;
use crate::rusk_http_client::RuskHttpClient;
use crate::utils::{RuskUtils, POSEIDON_TREE_DEPTH, TRANSFER_CONTRACT_STR, TRANSFER_CONTRACT};
use crate::Error;
use dusk_bytes::Serializable;
use execution_core::transfer::{AccountData, TreeLeaf};
use execution_core::{BlsPublicKey, BlsScalar, Note, ViewKey};
use futures::StreamExt;
use poseidon_merkle::Opening as PoseidonOpening;
use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, RwLock};
use tracing::info;
use zk_citadel_moat::{ContractInquirer, StreamAux};
use wallet::StateClient;

#[derive(Debug)]
pub struct DCliStateClient {
    pub client: RuskHttpClient,
    pub cache: Arc<RwLock<HashMap<Vec<u8>, DummyCacheItem>>>,
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
    pub fn new(rusk_http_client: RuskHttpClient) -> Self {
        let cache = Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
        Self {
            client: rusk_http_client,
            cache,
        }
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

        info!("Requesting notes from height {}", vk_cache.last_height);
        // let vk_bytes = vk.to_bytes();

        // let stream = RuskUtils::default()
        //     .get_notes(vk_bytes.as_ref(), vk_cache.last_height)
        //     .wait()?;
        //
        // let response_notes = stream.collect::<Vec<(Note, u64)>>().wait();
        let mut response_notes = Vec::new();
        let stream = ContractInquirer::query_contract_with_feeder(
            rusk_http_client,
            vk_cache.last_height,
            TRANSFER_CONTRACT,
            "leaves_from_height"
        )
        .wait()?;
        const ITEM_LEN: usize = mem::size_of::<(u64, Note)>();
        StreamAux::find_items::<(u64, Vec<u8>), ITEM_LEN>(
            |leaf|{
                let leaf = rkyv::from_bytes::<TreeLeaf>(&bytes)
                    .expect("The contract should always return valid leaves");
                if vk.owns(leaf.stealth_address()) {
                    response_notes.push((leaf.block_height, leaf.note))
                }
            },
            stream,
        )?;

        for (note, block_height) in response_notes {
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
        _nullifiers: &[BlsScalar],
    ) -> Result<Vec<BlsScalar>, Self::Error> {
        Ok(vec![])
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
        let account = RuskUtils::default().account(pk)?;
        Ok(account)
    }
}
