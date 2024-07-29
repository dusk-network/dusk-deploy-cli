// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Error;
use execution_core::transfer::{ContractExec, Fee, Transaction};
use execution_core::{PublicKey, SecretKey};

pub struct TxBuilder;

impl TxBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        // pub fn phoenix_execute<Rng>
        // self is a wallet with store, state, prover
        // TestStore, TestStateClient, TestProverClient
        // TestStore just provides get_seed, implements Store which gives "fetch_secret_key"
        // TestStateClient provides Rusk - let's see what from Rusk it needs
        // TestProverClient just provides prover
        &self,
        rng: &mut Rng,
        exec: impl Into<ContractExec>,
        sender_index: u64,
        gas_limit: u64,
        gas_price: u64,
        deposit: u64,
    ) -> Result<Transaction, Error> {
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

    #[allow(clippy::too_many_arguments)]
    fn phoenix_transaction<Rng, MaybeExec>(
        &self, // self is a wallet with store, state, prover
        // providing methods:
        // - inputs_and_change_output <-- imp.rs
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

        let (inputs, outputs) = self.inputs_and_change_output(
            rng,
            &sender_sk,
            &sender_pk,
            &receiver_pk,
            value,
            gas_limit * gas_price,
            deposit,
        )?;

        let fee = Fee::new(rng, &sender_pk, gas_limit, gas_price);
        let contract_call =
            exec.maybe_phoenix_exec(rng, inputs.iter().map(|(n, _, _)| n.clone()).collect());

        let utx = new_unproven_tx(
            // global method in imp.rs
            rng,
            &self.state,
            &sender_sk,
            inputs,
            outputs,
            fee,
            deposit,
            contract_call,
        )
        .map_err(Error::from_state_err)?;

        self.prover
            .compute_proof_and_propagate(&utx)
            .map_err(Error::from_prover_err)
    }

    fn compute_proof_and_propagate(
        &self,
        utx: &UnprovenTransaction,
    ) -> Result<Transaction, Self::Error> {
        let utx_bytes = &utx.to_var_bytes()[..];
        let proof = self.prover.prove_execute(utx_bytes)?;
        info!("UTX: {}", hex::encode(utx_bytes));
        let proof = Proof::from_slice(&proof).map_err(Error::Serialization)?;
        let tx = utx.clone().gen_transaction(proof);

        //Propagate is not required yet

        Ok(tx.into())
    }
}
