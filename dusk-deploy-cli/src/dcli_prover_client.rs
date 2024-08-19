use crate::block::Block;
use crate::Error;
use dusk_bytes::DeserializableSlice;
use dusk_plonk::prelude::Proof;
use execution_core::transfer::MoonlightTransaction;
use execution_core::{transfer::Transaction, BlsScalar};
use rusk_http_client::{BlockchainInquirer, RuskHttpClient, RuskRequest};
use rusk_prover::UnprovenTransaction;
use std::borrow::Cow;
use std::fmt::Debug;
use std::thread;
use tracing::info;

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
            status: |a| info!("{}", a),
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
        self.status("Proof success!");
        let proof = Proof::from_slice(&proof_bytes)?;
        let tx = utx.clone().gen_transaction(proof);
        let tx = Transaction::Phoenix(tx);
        let tx_bytes = tx.to_var_bytes();

        self.status("Attempt to preverify tx...");
        let preverify_req = RuskRequest::new("preverify", tx_bytes.clone());
        let _ = self.state.call(2, "rusk", &preverify_req).wait()?;
        self.status("Preverify success!");

        self.status("Propagating tx...");
        let propagate_req = RuskRequest::new("propagate_tx", tx_bytes);
        let _ = self.state.call(2, "Chain", &propagate_req).wait()?;
        self.status("Transaction propagated!");

        let tx_id = BlsScalar::hash_to_scalar(tx.to_hash_input_bytes().as_slice());
        let tx_id_str = hex::encode(tx_id.to_bytes());
        info!("Transaction id = {}", tx_id_str);
        for _ in 0..20 {
            let r = BlockchainInquirer::retrieve_tx_err(tx_id_str.clone(), &self.state).wait();
            if r.is_ok() {
                return match r.unwrap() {
                    Some(err) => Err(Error::Deploy(Cow::from(err))),
                    None => Ok(tx),
                };
            }
            thread::sleep(std::time::Duration::from_secs(3));
        }
        Err(Error::Propagate("Transaction timed out".into()))
    }

    fn propagate_moonlight_transaction(
        &self,
        tx: &MoonlightTransaction,
    ) -> Result<Transaction, Self::Error> {
        let tx = Transaction::Moonlight(tx.clone());
        let tx_bytes = tx.to_var_bytes();

        self.status("Attempt to preverify tx...");
        let preverify_req = RuskRequest::new("preverify", tx_bytes.clone());
        let _ = self.state.call(2, "rusk", &preverify_req).wait()?;
        self.status("Preverify success!");

        self.status("Propagating tx...");
        let propagate_req = RuskRequest::new("propagate_tx", tx_bytes);
        let _ = self.state.call(2, "Chain", &propagate_req).wait()?;
        self.status("Transaction propagated!");

        let tx_id = BlsScalar::hash_to_scalar(tx.to_hash_input_bytes().as_slice());
        let tx_id_str = hex::encode(tx_id.to_bytes());
        info!("Transaction id = {}", tx_id_str);
        for _ in 0..20 {
            let r = BlockchainInquirer::retrieve_tx_err(tx_id_str.clone(), &self.state).wait();
            if r.is_ok() {
                return match r.unwrap() {
                    Some(err) => Err(Error::Deploy(Cow::from(err))),
                    None => Ok(tx),
                };
            }
            thread::sleep(std::time::Duration::from_secs(3));
        }
        Err(Error::Propagate("Transaction timed out".into()))
    }
}

impl DCliProverClient {
    fn status(&self, text: &str) {
        (self.status)(text)
    }
}
