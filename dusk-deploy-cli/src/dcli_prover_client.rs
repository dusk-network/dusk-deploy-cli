use crate::Error;
use dusk_bytes::DeserializableSlice;
use dusk_plonk::prelude::Proof;
use execution_core::transfer::Transaction;
use rusk_prover::{LocalProver, Prover, UnprovenTransaction};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Default)]
pub struct DCliProverClient {
    pub prover: LocalProver,
}

impl Debug for DCliProverClient {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl wallet::ProverClient for DCliProverClient {
    type Error = Error;
    /// Requests that a node prove the given transaction and later propagates it
    fn compute_proof_and_propagate(
        &self,
        utx: &UnprovenTransaction,
    ) -> Result<Transaction, Self::Error> {
        let utx_bytes = &utx.to_var_bytes()[..];
        let proof = self.prover.prove_execute(utx_bytes)?;
        let proof = Proof::from_slice(&proof).map_err(|e| Error::Serialization(Arc::from(e)))?;
        let tx = utx.clone().gen_transaction(proof);

        //Propagate is not required yet

        Ok(tx.into())
    }
}
