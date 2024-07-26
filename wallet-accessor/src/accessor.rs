// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bls12_381::BlsScalar;
use dusk_wallet::dat::{read_file_version, DatFileVersion};
use dusk_wallet::gas::Gas;
use dusk_wallet::{DecodedNote, Error, SecureWalletFile, Wallet, WalletPath};
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;
use sha2::{Digest, Sha256};
use tracing::debug;
use crate::BlockchainAccessConfig;
use crate::Password::{Pwd, PwdHash};

pub const MAX_CALL_SIZE: usize = 65536;

#[derive(Debug, Clone)]
pub enum Password {
    Pwd(String),
    PwdHash(String),
}

#[derive(Debug)]
pub struct WalletAccessor {
    pub path: WalletPath,
    pub pwd: Password,
    pub pwd_bytes: Vec<u8>,
}

impl SecureWalletFile for WalletAccessor {
    fn path(&self) -> &WalletPath {
        &self.path
    }

    fn pwd(&self) -> &[u8] {
        self.pwd_bytes.as_slice()
    }
}

impl WalletAccessor {
    pub fn create(
        wallet_path: WalletPath,
        pwd: Password,
    ) -> Result<Self, Error> {
        let dat_file_version = read_file_version(&wallet_path)?;
        let is_sha256 =
            matches!(dat_file_version, DatFileVersion::RuskBinaryFileFormat(_));
        Ok(Self {
            path: wallet_path,
            pwd: pwd.clone(),
            pwd_bytes: {
                match &pwd {
                    Pwd(s) => {
                        if is_sha256 {
                            let mut hasher = Sha256::new();
                            hasher.update(s.as_bytes());
                            hasher.finalize().to_vec()
                        } else {
                            let hash = blake3::hash(s.as_bytes());
                            hash.as_bytes().to_vec()
                        }
                    }
                    PwdHash(h) => hex::decode(h.as_str())
                        .expect("Password hash should be valid hex string")
                        .to_vec(),
                }
            },
        })
    }

    async fn get_wallet(
        &self,
        cfg: &BlockchainAccessConfig,
    ) -> Result<Wallet<WalletAccessor>, dusk_wallet::Error> {
        let wallet_accessor =
            WalletAccessor::create(self.path.clone(), self.pwd.clone())?;
        let mut wallet = Wallet::from_file(wallet_accessor)?;
        wallet
            .connect_with_status(
                cfg.rusk_address.clone(),
                cfg.prover_address.clone(),
                |s| {
                    debug!(target: "wallet", "{s}",);
                },
            )
            .await?;
        wallet.sync().await?;
        assert!(wallet.is_online().await, "Wallet should be online");
        Ok(wallet)
    }

    /// submits a transaction which will execute a given method
    /// of a given contract
    pub async fn execute_contract_method<C>(
        &self,
        data: C,
        contract_id: ModuleId,
        call_name: String,
        cfg: &BlockchainAccessConfig,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<BlsScalar, dusk_wallet::Error>
        where
            C: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
    {
        let wallet = self.get_wallet(cfg).await?;

        debug!(
            "Sending tx with a call to method '{}' of contract='{}'",
            call_name.clone(),
            hex::encode(contract_id)
        );

        let sender = wallet.default_address();
        let mut gas = Gas::new(gas_limit);
        gas.set_price(gas_price);

        let tx = wallet
            .execute(sender, contract_id, call_name.clone(), data, gas)
            .await?;
        let tx_id = rusk_abi::hash::Hasher::digest(tx.to_hash_input_bytes());
        Ok(tx_id)
    }

    /// provides hashes of all notes belonging to the default address
    pub async fn get_notes(
        &self,
        cfg: &BlockchainAccessConfig,
    ) -> Result<Vec<DecodedNote>, dusk_wallet::Error> {
        let wallet = self.get_wallet(cfg).await?;
        wallet.get_all_notes(wallet.default_address()).await
    }
}
