// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;

use std::pin::Pin;
use std::sync::{mpsc, Arc};

use dusk_bytes::DeserializableSlice;
use futures::Stream;
use tokio::spawn;
use tracing::{error, info};

use execution_core::transfer::{AccountData, ContractId, TreeLeaf, TRANSFER_TREE_DEPTH};
use execution_core::{BlsPublicKey, BlsScalar, Note, ViewKey};
use poseidon_merkle::Opening as PoseidonOpening;
// use rusk_abi::{ContractId, STAKE_CONTRACT, TRANSFER_CONTRACT, VM};
use bytecheck::CheckBytes;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{
    ser::serializers::{BufferScratch, BufferSerializer, CompositeSerializer},
    Archive, Deserialize, Infallible, Serialize,
};

pub type StoredNote = (Note, u64);

pub type GetNotesStream = Pin<Box<dyn Stream<Item = StoredNote> + Send>>;

pub const TRANSFER_CONTRACT_STR: &str =
    "0100000000000000000000000000000000000000000000000000000000000000"; // todo

pub const CONTRACT_ID_BYTES: usize = 32;

#[inline]
const fn reserved(b: u8) -> ContractId {
    let mut bytes = [0u8; CONTRACT_ID_BYTES];
    bytes[0] = b;
    bytes
}

/// ID of the genesis transfer contract
pub const TRANSFER_CONTRACT: ContractId = reserved(0x1);

pub const POSEIDON_TREE_DEPTH: usize = 17; // todo

pub const SCRATCH_BUF_BYTES: usize = 64; // todo

pub type StandardBufSerializer<'a> = CompositeSerializer<
    BufferSerializer<&'a mut [u8]>,
    BufferScratch<&'a mut [u8; SCRATCH_BUF_BYTES]>,
>;

#[derive(Clone, Default)]
pub struct RuskUtils;

impl RuskUtils {
    /// Performs a feeder query returning the leaves of the transfer tree
    /// starting from the given height. The function will block while executing,
    /// and the results of the query will be passed through the `receiver`
    /// counterpart of the given `sender`.
    ///
    /// The receiver of the leaves is responsible for deserializing the leaves
    /// appropriately - i.e. using `rkyv`.
    pub fn leaves_from_height(
        &self,
        height: u64,
        sender: mpsc::Sender<Vec<u8>>,
    ) -> Result<(), Error> {
        self.feeder_query(
            TRANSFER_CONTRACT,
            "leaves_from_height",
            &height,
            sender,
            None,
        )
    }

    /// Returns the root of the transfer tree.
    pub fn tree_root(&self) -> Result<BlsScalar, Error> {
        info!("Received tree_root request");
        self.query(TRANSFER_CONTRACT, "root", &())
    }

    /// Returns the opening of the transfer tree at the given position.
    pub fn tree_opening(
        &self,
        pos: u64,
    ) -> Result<Option<PoseidonOpening<(), TRANSFER_TREE_DEPTH>>, Error> {
        self.query(TRANSFER_CONTRACT, "opening", &pos)
    }

    pub async fn get_notes(&self, vk: &[u8], height: u64) -> Result<GetNotesStream, Error> {
        info!("Received GetNotes request");

        let vk = match vk.is_empty() {
            false => {
                let vk = ViewKey::from_slice(vk).map_err(|e| Error::Serialization(Arc::from(e)))?;
                Some(vk)
            }
            true => None,
        };

        let (sender, receiver) = mpsc::channel();

        // Clone this and move it to the thread
        let rusk = self.clone();

        // Spawn a task responsible for running the feeder query.
        spawn(async move {
            if let Err(err) = rusk.leaves_from_height(height, sender) {
                error!("GetNotes errored: {err}");
            }
        });

        // Make a stream from the receiver and map the elements to be the
        // expected output
        let stream = tokio_stream::iter(receiver.into_iter().filter_map(move |bytes| {
            let leaf = rkyv::from_bytes::<TreeLeaf>(&bytes)
                .expect("The contract should always return valid leaves");
            match &vk {
                Some(vk) => vk
                    .owns(leaf.note.stealth_address())
                    .then_some((leaf.note, leaf.block_height)),
                None => Some((leaf.note, leaf.block_height)),
            }
        }));

        Ok(Box::pin(stream) as GetNotesStream)
    }

    fn feeder_query<A>(
        &self,
        contract_id: ContractId,
        call_name: &str,
        call_arg: &A,
        feeder: mpsc::Sender<Vec<u8>>,
        base_commit: Option<[u8; 32]>,
    ) -> Result<(), Error>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> bytecheck::CheckBytes<DefaultValidator<'b>>,
    {
        // For queries we set a point limit of effectively infinite and a block
        // height of zero since this doesn't affect the result.
        let mut session = self.session(0, base_commit)?;

        session.feeder_call::<_, ()>(
            contract_id,
            call_name,
            call_arg,
            self.feeder_gas_limit,
            feeder,
        )?;

        Ok(())
    }

    pub(crate) fn query<A, R>(
        &self,
        contract_id: ContractId,
        call_name: &str,
        call_arg: &A,
    ) -> Result<R, Error>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> bytecheck::CheckBytes<DefaultValidator<'b>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible> + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let mut results = Vec::with_capacity(1);
        self.query_seq(contract_id, call_name, call_arg, |r| {
            results.push(r);
            None
        })?;
        Ok(results.pop().unwrap())
    }

    fn query_seq<A, R, F>(
        &self,
        contract_id: ContractId,
        call_name: &str,
        call_arg: &A,
        mut closure: F,
    ) -> Result<(), Error>
    where
        F: FnMut(R) -> Option<A>,
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> bytecheck::CheckBytes<DefaultValidator<'b>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible> + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        // For queries we set a point limit of effectively infinite and a block
        // height of zero since this doesn't affect the result.
        let mut session = self.session(0, None)?;

        let mut result = session
            .call(contract_id, call_name, call_arg, u64::MAX)?
            .data;

        while let Some(call_arg) = closure(result) {
            result = session
                .call(contract_id, call_name, &call_arg, u64::MAX)?
                .data;
        }

        session.call::<_, ()>(contract_id, call_name, call_arg, u64::MAX)?;

        Ok(())
    }

    /// Returns an account's information.
    pub fn account(&self, pk: &BlsPublicKey) -> Result<AccountData, Error> {
        self.query(TRANSFER_CONTRACT, "account", pk)
    }
}
