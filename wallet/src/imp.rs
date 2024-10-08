// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{BalanceInfo, PhoenixTransaction, ProverClient, StateClient, Store, MAX_CALL_SIZE};

use core::convert::Infallible;

use alloc::string::FromUtf8Error;
use alloc::vec::Vec;
use std::mem;

use dusk_bytes::{Error as BytesError, Serializable};
use execution_core::transfer::phoenix::{NoteOpening, Prove, TxCircuitVec};
use execution_core::{
    signatures::bls::{PublicKey as BlsPublicKey, SecretKey as BlsSecretKey},
    transfer::{
        data::{ContractCall, ContractDeploy, TransactionData},
        moonlight::{AccountData, Transaction as MoonlightTransaction},
        phoenix::{Note, PublicKey, SecretKey, ViewKey},
        Transaction,
    },
    BlsScalar, JubJubScalar,
};
use ff::Field;
use phoenix_core::OUTPUT_NOTES;
use rand_core::{CryptoRng, Error as RngError, RngCore};
use rkyv::ser::serializers::{
    AllocScratchError, CompositeSerializerError, SharedSerializeMapError,
};
use rkyv::validation::validators::CheckDeserializeError;

const MAX_INPUT_NOTES: usize = 4;

type SerializerError =
    CompositeSerializerError<Infallible, AllocScratchError, SharedSerializeMapError>;

/// The error type returned by this crate.
#[derive(thiserror::Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Error<S: Store, SC: StateClient, PC: ProverClient> {
    /// Underlying store error.
    #[error(transparent)]
    Store(S::Error),
    /// Error originating from the state client.
    #[error(transparent)]
    State(SC::Error),
    /// Error originating from the prover client.
    #[error(transparent)]
    Prover(PC::Error),
    /// Rkyv serialization.
    #[error("Serialization error")]
    Rkyv,
    /// Random number generator error.
    #[error(transparent)]
    Rng(RngError),
    /// Serialization and deserialization of Dusk types.
    #[error("Bytes error")]
    Bytes(BytesError),
    /// Bytes were meant to be utf8 but aren't.
    #[error(transparent)]
    Utf8(FromUtf8Error),
    /// Originating from the transaction model.
    #[error("Transaction model error occurred: {0}")]
    Phoenix(execution_core::Error),
    /// Originating from Phoenix Core.
    #[error("Phoenix core error occurred: {0}")]
    PhoenixCore(phoenix_core::Error),
    /// Not enough balance to perform transaction.
    #[error("Not enough balance")]
    NotEnoughBalance,
    /// Note combination for the given value is impossible given the maximum
    /// amount if inputs in a transaction.
    #[error("Note combination problem")]
    NoteCombinationProblem,
}

impl<S: Store, SC: StateClient, PC: ProverClient> Error<S, SC, PC> {
    /// Returns an error from the underlying store error.
    pub fn from_store_err(se: S::Error) -> Self {
        Self::Store(se)
    }
    /// Returns an error from the underlying state client.
    pub fn from_state_err(se: SC::Error) -> Self {
        Self::State(se)
    }
    /// Returns an error from the underlying prover client.
    pub fn from_prover_err(pe: PC::Error) -> Self {
        Self::Prover(pe)
    }
}

impl<S: Store, SC: StateClient, PC: ProverClient> From<SerializerError> for Error<S, SC, PC> {
    fn from(_: SerializerError) -> Self {
        Self::Rkyv
    }
}

impl<C, D, S: Store, SC: StateClient, PC: ProverClient> From<CheckDeserializeError<C, D>>
    for Error<S, SC, PC>
{
    fn from(_: CheckDeserializeError<C, D>) -> Self {
        Self::Rkyv
    }
}

impl<S: Store, SC: StateClient, PC: ProverClient> From<RngError> for Error<S, SC, PC> {
    fn from(re: RngError) -> Self {
        Self::Rng(re)
    }
}

impl<S: Store, SC: StateClient, PC: ProverClient> From<BytesError> for Error<S, SC, PC> {
    fn from(be: BytesError) -> Self {
        Self::Bytes(be)
    }
}

impl<S: Store, SC: StateClient, PC: ProverClient> From<FromUtf8Error> for Error<S, SC, PC> {
    fn from(err: FromUtf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl<S: Store, SC: StateClient, PC: ProverClient> From<execution_core::Error> for Error<S, SC, PC> {
    fn from(pe: execution_core::Error) -> Self {
        Self::Phoenix(pe)
    }
}

/// A wallet implementation.
///
/// This is responsible for holding the keys, and performing operations like
/// creating transactions.
pub struct Wallet<S, SC, PC> {
    store: S,
    state: SC,
    prover: PC,
}

impl<S, SC, PC> Wallet<S, SC, PC> {
    /// Create a new wallet given the underlying store and node client.
    pub const fn new(store: S, state: SC, prover: PC) -> Self {
        Self {
            store,
            state,
            prover,
        }
    }

    /// Return the inner Store reference
    pub const fn store(&self) -> &S {
        &self.store
    }

    /// Return the inner State reference
    pub const fn state(&self) -> &SC {
        &self.state
    }

    /// Return the inner Prover reference
    pub const fn prover(&self) -> &PC {
        &self.prover
    }
}

struct DummyProver();

impl Prove for DummyProver {
    fn prove(&self, tx_circuit_vec_bytes: &[u8]) -> Result<Vec<u8>, execution_core::Error> {
        Ok(TxCircuitVec::from_slice(tx_circuit_vec_bytes)
            .expect("serialization should be ok")
            .to_var_bytes()
            .to_vec())
    }
}

impl<S, SC, PC> Wallet<S, SC, PC>
where
    S: Store,
    SC: StateClient,
    PC: ProverClient,
{
    /// Retrieve the public key with the given index.
    pub fn public_key(&self, index: u64) -> Result<PublicKey, Error<S, SC, PC>> {
        self.store
            .fetch_secret_key(index)
            .map(|sk| PublicKey::from(&sk))
            .map_err(Error::from_store_err)
    }

    /// Retrieve the account public key with the given index.
    pub fn account_public_key(&self, index: u64) -> Result<BlsPublicKey, Error<S, SC, PC>> {
        self.store
            .fetch_account_secret_key(index)
            .map(|stake_sk| From::from(&stake_sk))
            .map_err(Error::from_store_err)
    }

    /// Fetches the notes and nullifiers in the state and returns the notes that
    /// are still available for spending.
    fn unspent_notes(&self, sk: &SecretKey) -> Result<Vec<Note>, Error<S, SC, PC>> {
        const CHUNK_SIZE: usize = MAX_CALL_SIZE / (8 * mem::size_of::<BlsScalar>());
        let vk = ViewKey::from(sk);

        let notes = self.state.fetch_notes(&vk).map_err(Error::from_state_err)?;

        // this part is a performance bottleneck and needs caching
        let nullifiers: Vec<_> = notes.iter().map(|(n, _)| n.gen_nullifier(sk)).collect();

        let mut existing_nullifiers: Vec<BlsScalar> = vec![];
        for chunk in nullifiers.chunks(CHUNK_SIZE) {
            existing_nullifiers.extend(
                self.state
                    .fetch_existing_nullifiers(chunk)
                    .map_err(Error::from_state_err)?,
            );
        }

        let unspent_notes = notes
            .into_iter()
            .zip(nullifiers)
            .filter(|(_, nullifier)| !existing_nullifiers.contains(nullifier))
            .map(|((note, _), _)| note)
            .collect();

        Ok(unspent_notes)
    }

    /// Here we fetch the notes and perform a "minimum number of notes
    /// required" algorithm to select which ones to use for this TX. This is
    /// done by picking notes largest to smallest until they combined have
    /// enough accumulated value.
    ///
    /// We also return the outputs with a possible change note (if applicable).
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn inputs_and_change_output<Rng: RngCore + CryptoRng>(
        &self,
        rng: &mut Rng,
        sender_sk: &SecretKey,
        sender_pk: &PublicKey,
        receiver_pk: &PublicKey,
        transfer_value: u64,
        max_fee: u64,
        deposit: u64,
    ) -> Result<
        (
            Vec<(Note, u64, JubJubScalar)>,
            [(Note, u64, JubJubScalar, [JubJubScalar; 2]); OUTPUT_NOTES],
        ),
        Error<S, SC, PC>,
    > {
        let notes = self.unspent_notes(sender_sk)?;
        let mut notes_and_values = Vec::with_capacity(notes.len());

        let sender_vk = ViewKey::from(sender_sk);

        let mut accumulated_value = 0;
        for note in notes.into_iter() {
            let val = note.value(Some(&sender_vk)).map_err(Error::PhoenixCore)?;
            let value_blinder = note
                .value_blinder(Some(&sender_vk))
                .map_err(Error::PhoenixCore)?;

            accumulated_value += val;
            notes_and_values.push((note, val, value_blinder));
        }

        if accumulated_value < transfer_value + max_fee {
            return Err(Error::NotEnoughBalance);
        }

        let inputs = pick_notes(transfer_value + max_fee + deposit, notes_and_values);

        if inputs.is_empty() {
            return Err(Error::NoteCombinationProblem);
        }

        let (transfer_note, transfer_value_blinder, transfer_sender_blinder) =
            generate_obfuscated_note(rng, sender_pk, receiver_pk, transfer_value);

        let change = inputs.iter().map(|v| v.1).sum::<u64>() - transfer_value - max_fee - deposit;
        let change_sender_blinder = [
            JubJubScalar::random(&mut *rng),
            JubJubScalar::random(&mut *rng),
        ];
        let change_note =
            Note::transparent(rng, sender_pk, sender_pk, change, change_sender_blinder);

        let outputs = [
            (
                transfer_note,
                transfer_value,
                transfer_value_blinder,
                transfer_sender_blinder,
            ),
            (
                change_note,
                change,
                JubJubScalar::zero(),
                change_sender_blinder,
            ),
        ];

        Ok((inputs, outputs))
    }

    #[allow(clippy::too_many_arguments)]
    fn phoenix_transaction<Rng, MaybeExec>(
        &self,
        rng: &mut Rng,
        sender_sk: &SecretKey,
        receiver_pk: &PublicKey,
        value: u64,
        gas_limit: u64,
        gas_price: u64,
        deposit: u64,
        exec: MaybeExec,
    ) -> Result<Transaction, Error<S, SC, PC>>
    where
        Rng: RngCore + CryptoRng,
        MaybeExec: MaybePhoenixExec<Rng>,
    {
        let sender_pk = PublicKey::from(sender_sk);

        let (inputs, _outputs) = self.inputs_and_change_output(
            rng,
            sender_sk,
            &sender_pk,
            receiver_pk,
            value,
            gas_limit * gas_price,
            deposit,
        )?;

        let inputs: Result<Vec<(Note, NoteOpening)>, SC::Error> = inputs
            .into_iter()
            .map(|(note, _, _)| {
                let opening = self.state.fetch_opening(&note)?;
                Ok::<(Note, NoteOpening), SC::Error>((note.clone(), opening))
            })
            .collect();
        let inputs: Vec<(Note, NoteOpening)> = inputs.map_err(Error::from_state_err)?;

        let contract_call =
            exec.maybe_phoenix_exec(rng, inputs.iter().map(|(n, _)| n.clone()).collect());

        let root = self
            .state
            .fetch_anchor()
            .map_err(|e| Error::from_state_err(e))?;
        let chain_id = self
            .state
            .fetch_chain_id()
            .map_err(|e| Error::from_state_err(e))?;

        let dummy_prover = DummyProver();
        let utx = PhoenixTransaction::new::<Rng, DummyProver>(
            rng,
            &sender_sk,
            &sender_pk,
            &receiver_pk,
            inputs,
            root,
            0,
            true,
            deposit,
            gas_limit,
            gas_price,
            chain_id,
            contract_call,
            &dummy_prover,
        )?;

        self.prover
            .compute_proof_and_propagate(&utx)
            .map_err(Error::from_prover_err)
    }

    #[allow(clippy::too_many_arguments)]
    fn moonlight_transaction(
        &self,
        from_sk: &BlsSecretKey,
        to: Option<BlsPublicKey>,
        value: u64,
        deposit: u64,
        gas_limit: u64,
        gas_price: u64,
        nonce: u64,
        chain_id: u8,
        exec: Option<impl Into<TransactionData>>,
    ) -> Result<Transaction, Error<S, SC, PC>> {
        let mt = MoonlightTransaction::new(
            from_sk,
            to,
            value,
            deposit,
            gas_limit,
            gas_price,
            nonce,
            chain_id,
            exec.map(Into::into),
        )?;

        self.prover
            .propagate_moonlight_transaction(&mt)
            .map_err(Error::<S, SC, PC>::from_prover_err) // todo: naming should no longer be about prover
    }

    /// Execute a generic contract call or deployment, using Phoenix notes to
    /// pay for gas.
    #[allow(clippy::too_many_arguments)]
    pub fn phoenix_execute<Rng>(
        &self,
        rng: &mut Rng,
        exec: impl Into<TransactionData>,
        sender_index: u64,
        gas_limit: u64,
        gas_price: u64,
        deposit: u64,
    ) -> Result<Transaction, Error<S, SC, PC>>
    where
        Rng: RngCore + CryptoRng,
    {
        let sender_sk = self
            .store
            .fetch_secret_key(sender_index)
            .map_err(Error::from_store_err)?;
        let receiver_pk = PublicKey::from(&sender_sk);

        self.phoenix_transaction(
            rng,
            &sender_sk,
            &receiver_pk,
            0,
            gas_limit,
            gas_price,
            deposit,
            exec.into(),
        )
    }

    /// Execute a generic contract call or deployment, using Moonlight to
    /// pay for gas.
    #[allow(clippy::too_many_arguments)]
    pub fn moonlight_execute(
        &self,
        exec: impl Into<TransactionData>,
        sender_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<Transaction, Error<S, SC, PC>> {
        let moonlight_sk: BlsSecretKey = self
            .store
            .fetch_account_secret_key(sender_index)
            .map_err(Error::from_store_err)?;
        let moonlight_pk = BlsPublicKey::from(&moonlight_sk);
        let acc_data = self
            .state
            .fetch_account(&moonlight_pk)
            .map_err(Error::from_state_err)?;
        let chain_id = self.state.fetch_chain_id().map_err(Error::from_state_err)?;

        println!(
            "account {} fetched: {:?}",
            bs58::encode(moonlight_pk.to_bytes()).into_string(),
            acc_data
        );

        let result = self.moonlight_transaction(
            &moonlight_sk,
            None,
            0,
            0,
            gas_limit,
            gas_price,
            acc_data.nonce + 1,
            chain_id,
            Some(exec.into()),
        );

        let acc_data = self
            .state
            .fetch_account(&moonlight_pk)
            .map_err(Error::from_state_err)?;

        println!(
            "account {} fetched: {:?}",
            bs58::encode(moonlight_pk.to_bytes()).into_string(),
            acc_data
        );

        result
    }

    /// Transfer Dusk in the form of Phoenix notes from one key to another.
    #[allow(clippy::too_many_arguments)]
    pub fn phoenix_transfer<Rng: RngCore + CryptoRng>(
        &self,
        rng: &mut Rng,
        sender_index: u64,
        receiver_pk: &PublicKey,
        value: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<Transaction, Error<S, SC, PC>> {
        let sender_sk = self
            .store
            .fetch_secret_key(sender_index)
            .map_err(Error::from_store_err)?;

        self.phoenix_transaction(
            rng,
            &sender_sk,
            receiver_pk,
            value,
            gas_limit,
            gas_price,
            0,
            None,
        )
    }

    /// Gets the balance of a key.
    pub fn get_balance(&self, sk_index: u64) -> Result<BalanceInfo, Error<S, SC, PC>> {
        let sender_sk = self
            .store
            .fetch_secret_key(sk_index)
            .map_err(Error::from_store_err)?;
        let vk = ViewKey::from(&sender_sk);

        let notes = self.unspent_notes(&sender_sk)?;
        let mut values = Vec::with_capacity(notes.len());

        for note in notes.into_iter() {
            values.push(note.value(Some(&vk)).map_err(Error::PhoenixCore)?);
        }
        values.sort_by(|a, b| b.cmp(a));

        let spendable = values.iter().take(MAX_INPUT_NOTES).sum();
        let value = spendable + values.iter().skip(MAX_INPUT_NOTES).sum::<u64>();

        Ok(BalanceInfo { value, spendable })
    }

    /// Gets the account data for a key.
    pub fn get_account(&self, sk_index: u64) -> Result<AccountData, Error<S, SC, PC>> {
        let account_sk = self
            .store
            .fetch_account_secret_key(sk_index)
            .map_err(Error::from_store_err)?;

        let account_pk = BlsPublicKey::from(&account_sk);

        let account = self
            .state
            .fetch_account(&account_pk)
            .map_err(Error::from_state_err)?;

        Ok(account)
    }
}

/// Optionally produces contract calls/executions for Phoenix transactions.
trait MaybePhoenixExec<R> {
    fn maybe_phoenix_exec(self, rng: &mut R, inputs: Vec<Note>) -> Option<TransactionData>;
}

impl<R> MaybePhoenixExec<R> for Option<TransactionData> {
    fn maybe_phoenix_exec(self, _rng: &mut R, _inputs: Vec<Note>) -> Option<TransactionData> {
        self
    }
}

impl<R> MaybePhoenixExec<R> for TransactionData {
    fn maybe_phoenix_exec(self, _rng: &mut R, _inputs: Vec<Note>) -> Option<TransactionData> {
        Some(self)
    }
}

impl<R> MaybePhoenixExec<R> for ContractCall {
    fn maybe_phoenix_exec(self, _rng: &mut R, _inputs: Vec<Note>) -> Option<TransactionData> {
        Some(TransactionData::Call(self))
    }
}

impl<R> MaybePhoenixExec<R> for ContractDeploy {
    fn maybe_phoenix_exec(self, _rng: &mut R, _inputs: Vec<Note>) -> Option<TransactionData> {
        Some(TransactionData::Deploy(self))
    }
}

impl<R, F, M> MaybePhoenixExec<R> for F
where
    F: FnOnce(&mut R, Vec<Note>) -> M,
    M: MaybePhoenixExec<R>,
{
    fn maybe_phoenix_exec(self, rng: &mut R, inputs: Vec<Note>) -> Option<TransactionData> {
        // NOTE: it may be (pun intended) possible to get rid of this clone if
        // we use a lifetime into a slice of `Note`s. However, it comes at the
        // cost of clarity. This is more important here, since this is testing
        // infrastructure, and not production code.
        let maybe = self(rng, inputs.clone());
        maybe.maybe_phoenix_exec(rng, inputs)
    }
}

/// Pick the notes to be used in a transaction from a vector of notes.
///
/// The notes are picked in a way to maximize the number of notes used, while
/// minimizing the value employed. To do this we sort the notes in ascending
/// value order, and go through each combination in a lexicographic order
/// until we find the first combination whose sum is larger or equal to
/// the given value. If such a slice is not found, an empty vector is returned.
///
/// Note: it is presupposed that the input notes contain enough balance to cover
/// the given `value`.
fn pick_notes(
    value: u64,
    notes_and_values: Vec<(Note, u64, JubJubScalar)>,
) -> Vec<(Note, u64, JubJubScalar)> {
    let mut notes_and_values = notes_and_values;
    let len = notes_and_values.len();

    if len <= MAX_INPUT_NOTES {
        return notes_and_values;
    }

    notes_and_values.sort_by(|(_, aval, _), (_, bval, _)| aval.cmp(bval));

    pick_lexicographic(notes_and_values.len(), |indices| {
        indices
            .iter()
            .map(|index| notes_and_values[*index].1)
            .sum::<u64>()
            >= value
    })
    .map(|indices| {
        indices
            .into_iter()
            .map(|index| notes_and_values[index].clone())
            .collect()
    })
    .unwrap_or_default()
}

fn pick_lexicographic<F: Fn(&[usize; MAX_INPUT_NOTES]) -> bool>(
    max_len: usize,
    is_valid: F,
) -> Option<[usize; MAX_INPUT_NOTES]> {
    let mut indices = [0; MAX_INPUT_NOTES];
    indices
        .iter_mut()
        .enumerate()
        .for_each(|(i, index)| *index = i);

    loop {
        if is_valid(&indices) {
            return Some(indices);
        }

        let mut i = MAX_INPUT_NOTES - 1;

        while indices[i] == i + max_len - MAX_INPUT_NOTES {
            if i > 0 {
                i -= 1;
            } else {
                break;
            }
        }

        indices[i] += 1;
        for j in i + 1..MAX_INPUT_NOTES {
            indices[j] = indices[j - 1] + 1;
        }

        if indices[MAX_INPUT_NOTES - 1] == max_len {
            break;
        }
    }

    None
}

/// Generates an obfuscated note for the given public spend key.
fn generate_obfuscated_note<Rng: RngCore + CryptoRng>(
    rng: &mut Rng,
    sender_pk: &PublicKey,
    receiver_pk: &PublicKey,
    value: u64,
) -> (Note, JubJubScalar, [JubJubScalar; 2]) {
    let value_blinder = JubJubScalar::random(&mut *rng);
    let sender_blinder = [
        JubJubScalar::random(&mut *rng),
        JubJubScalar::random(&mut *rng),
    ];

    (
        Note::obfuscated(
            rng,
            sender_pk,
            receiver_pk,
            value,
            value_blinder,
            sender_blinder,
        ),
        value_blinder,
        sender_blinder,
    )
}
