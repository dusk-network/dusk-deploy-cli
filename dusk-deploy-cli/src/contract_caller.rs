// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.


use dusk_bls12_381::BlsScalar;
use dusk_wallet::WalletPath;
use phoenix_core::transaction::ModuleId;
use rkyv::ser::serializers::AllocSerializer;

use crate::Error;
use wallet_accessor::{BlockchainAccessConfig, MAX_CALL_SIZE, Password, WalletAccessor};

pub struct ContractCaller;

impl ContractCaller {
    // pub fn call(
    //     wallet_path: &WalletPath,
    //     psw: &Password,
    //     blockchain_access_config: &BlockchainAccessConfig,
    //     gas_limit: u64,
    //     gas_price: u64,
    //     ssk: SecretSpendKey,
    // ) {
    //
    // }

    #[allow(clippy::too_many_arguments)]
    pub async fn execute_contract_method<P, M>(
        payload: P,
        cfg: &BlockchainAccessConfig,
        wallet_path: &WalletPath,
        password: &Password,
        gas_limit: u64,
        gas_price: u64,
        contract_id: ModuleId,
        method: M,
    ) -> Result<BlsScalar, Error>
        where
            P: rkyv::Serialize<AllocSerializer<MAX_CALL_SIZE>>,
            M: AsRef<str>,
    {
        let wallet_accessor =
            WalletAccessor::create(wallet_path.clone(), password.clone())?;
        let tx_id = wallet_accessor
            .execute_contract_method(
                payload,
                contract_id,
                method.as_ref().to_string(),
                cfg,
                gas_limit,
                gas_price,
            )
            .await?;
        Ok(tx_id)
    }

}
