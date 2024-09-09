// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::block::Block;
use crate::Error;
use dusk_bytes::Serializable;
use execution_core::transfer::phoenix::{NoteLeaf, NoteOpening};
use execution_core::{
    signatures::bls::PublicKey as BlsPublicKey,
    transfer::{
        moonlight::AccountData,
        phoenix::{Note, ViewKey},
    },
    BlsScalar, ContractId,
};
use rusk_http_client::RuskHttpClient;
use rusk_http_client::{ContractInquirer, StreamAux};
use std::cmp::{max, Ordering};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, RwLock};
use tracing::info;
use wallet::{EnrichedNote, StateClient};

const CONTRACT_ID_BYTES: usize = 32;

#[inline]
const fn reserved(b: u8) -> ContractId {
    let mut bytes = [0u8; CONTRACT_ID_BYTES];
    bytes[0] = b;
    ContractId::from_bytes(bytes)
}

const TRANSFER_CONTRACT: ContractId = reserved(0x1);

const TRANSFER_CONTRACT_STR: &str =
    "0100000000000000000000000000000000000000000000000000000000000000";

const ITEM_LEN: usize = mem::size_of::<NoteLeaf>();

#[derive(Debug, Clone)]
pub struct NoteBlockHeight(pub Note, pub u64);

impl NoteBlockHeight {
    pub fn as_enriched_note(&self) -> EnrichedNote {
        (self.0.clone(), self.1)
    }
}

impl PartialOrd<Self> for NoteBlockHeight {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.pos().cmp(other.0.pos()))
    }
}

impl Ord for NoteBlockHeight {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.pos().cmp(other.0.pos())
    }
}

impl PartialEq<Self> for NoteBlockHeight {
    fn eq(&self, other: &Self) -> bool {
        self.0.pos() == other.0.pos()
    }
}

impl Eq for NoteBlockHeight {}

pub struct DCliStateClient {
    pub client: RuskHttpClient,
    pub cache: Arc<RwLock<HashMap<Vec<u8>, DummyCacheItem>>>,
    pub start_block_height: u64,
}

#[derive(Default, Debug, Clone)]
pub struct DummyCacheItem {
    notes: BTreeSet<NoteBlockHeight>,
    last_height: u64,
}

impl DummyCacheItem {
    fn add(&mut self, enriched_note: NoteBlockHeight) {
        if !self.notes.contains(&enriched_note) {
            self.last_height = enriched_note.1;
            self.notes.insert(enriched_note);
        }
    }
}

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
            TRANSFER_CONTRACT.to_bytes(),
            "leaves_from_height",
        )
        .wait()?;
        StreamAux::find_items::<NoteLeaf, ITEM_LEN>(
            |leaf| {
                if vk.owns(leaf.note.stealth_address()) {
                    response_notes.push((leaf.note.clone(), leaf.block_height))
                }
            },
            &mut stream,
        )?;

        for note_block_height in response_notes {
            // Filter out duplicated notes and update the last
            vk_cache.add(NoteBlockHeight(note_block_height.0, note_block_height.1))
        }
        drop(cache_read);
        self.cache
            .write()
            .unwrap()
            .insert(vk.to_bytes().to_vec(), vk_cache.clone());

        Ok(vk_cache
            .notes
            .iter()
            .map(|nh| nh.as_enriched_note())
            .collect())
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
    fn fetch_opening(&self, note: &Note) -> Result<NoteOpening, Self::Error> {
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

    fn fetch_chain_id(&self) -> Result<u8, Error> {
        let data = self
            .client
            .contract_query::<_, { u8::SIZE }>(TRANSFER_CONTRACT_STR, "chain_id", &())
            .wait()?;

        let res: u8 = rkyv::from_bytes(&data).map_err(|_| Error::Rkyv)?;

        Ok(res)
    }
}
