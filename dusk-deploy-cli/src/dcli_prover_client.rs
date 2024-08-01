use crate::block::Block;
use crate::Error;
use dusk_bytes::DeserializableSlice;
use dusk_plonk::prelude::Proof;
use execution_core::transfer::Transaction;
use rusk_http_client::{RuskHttpClient, RuskRequest};
use rusk_prover::UnprovenTransaction;
use std::fmt::Debug;

pub struct DCliProverClient {
    state: RuskHttpClient,
    prover: RuskHttpClient,
    status: fn(status: &str),
}

impl Debug for DCliProverClient {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl DCliProverClient {
    pub fn new(state: RuskHttpClient, prover: RuskHttpClient) -> Self {
        DCliProverClient {
            state,
            prover,
            status: |_| {},
        }
    }

    /// Sets the callback method to send status updates
    #[allow(dead_code)]
    pub fn set_status_callback(&mut self, status: fn(&str)) {
        self.status = status;
    }

    #[allow(dead_code)]
    pub async fn check_connection(&self) -> Result<(), reqwest::Error> {
        self.state.check_connection().await?;
        self.prover.check_connection().await
    }
}

impl wallet::ProverClient for DCliProverClient {
    type Error = Error;
    /// Requests that a node prove the given transaction and later propagates it
    fn compute_proof_and_propagate(
        &self,
        utx: &UnprovenTransaction,
    ) -> Result<Transaction, Self::Error> {
        self.status("Proving tx, please wait...");
        let utx_bytes = utx.to_var_bytes();
        let prove_req = RuskRequest::new("prove_execute", utx_bytes);
        let proof_bytes = self.prover.call(2, "rusk", &prove_req).wait()?;
        // self.status("Proof success!");
        let proof = Proof::from_slice(&proof_bytes)?;
        let tx = utx.clone().gen_transaction(proof);
        let tx_bytes = tx.to_var_bytes();

        // self.status("Attempt to preverify tx...");
        let preverify_req = RuskRequest::new("preverify", tx_bytes.clone());
        let _ = self.state.call(2, "rusk", &preverify_req).wait()?;
        // self.status("Preverify success!");

        // self.status("Propagating tx...");
        let propagate_req = RuskRequest::new("propagate_tx", tx_bytes);
        let _ = self.state.call(2, "Chain", &propagate_req).wait()?;
        // self.status("Transaction propagated!");

        Ok(tx.into())
    }
}

impl DCliProverClient {
    fn status(&self, text: &str) {
        (self.status)(text)
    }
}
