// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::executor::Executor;
use crate::gen_contract_id;
use crate::{DCliProverClient, DCliStateClient, DCliStore, Error};
use piecrust_uplink::ContractId;
use wallet::Wallet;

pub struct ContractDeployer;

impl ContractDeployer {
    /// Returns (true, contract id) if contract already existed
    /// Returns (false, contract id) if contract was successfully deployed
    pub fn deploy(
        wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
        bytecode: &[u8],
        owner: &[u8],
        wallet_index: u64,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
        deploy_data: &[u8],
    ) -> Result<(bool, Option<ContractId>), Error> {
        let constructor_args = Some(deploy_data.to_vec());

        let result = Executor::deploy_via_moonlight(
            &wallet,
            &bytecode,
            &owner,
            constructor_args,
            nonce,
            wallet_index,
            gas_limit,
            gas_price,
        );

        let contract_id = Self::get_contract_id(bytecode, nonce, &owner);
        match result {
            Ok(()) => Ok((false, Some(contract_id))),
            ref r @ Err(ref e) => {
                if !format!("{:?}", e).contains("already exists") {
                    r.clone().map(|_| (false, None))
                } else {
                    Ok((true, Some(contract_id)))
                }
            }
        }
    }

    pub fn get_contract_id(bytecode: &[u8], nonce: u64, owner: &[u8]) -> ContractId {
        ContractId::from(gen_contract_id(bytecode, nonce, &owner))
    }
}
