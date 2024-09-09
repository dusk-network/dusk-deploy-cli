// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::dcli_store::DCliStore;
use execution_core::transfer::data::{
    ContractBytecode, ContractCall, ContractDeploy, TransactionData,
};
use rand::prelude::*;
use rand::rngs::StdRng;
use rusk_http_client::ContractId;
use wallet::Wallet;

use crate::Error;

fn bytecode_hash(bytecode: impl AsRef<[u8]>) -> [u8; 32] {
    let hash = blake3::hash(bytecode.as_ref());
    hash.into()
}

pub struct Executor;

impl Executor {
    #[allow(clippy::too_many_arguments)]
    pub fn deploy_via_phoenix(
        wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
        bytecode: &Vec<u8>,
        owner: &[u8],
        init_args: Option<Vec<u8>>,
        nonce: u64,
        wallet_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<(), Error> {
        let mut rng = StdRng::seed_from_u64(0xcafe);
        let hash = bytecode_hash(bytecode.as_slice());
        wallet.phoenix_execute(
            &mut rng,
            TransactionData::Deploy(ContractDeploy {
                bytecode: ContractBytecode {
                    hash,
                    bytes: bytecode.clone(),
                },
                owner: owner.to_vec(),
                init_args,
                nonce,
            }),
            wallet_index,
            gas_limit,
            gas_price,
            0u64,
        )?;

        Ok(())
    }

    // #[allow(clippy::too_many_arguments)]
    // pub fn deploy_via_moonlight(
    //     wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
    //     bytecode: &Vec<u8>,
    //     owner: &[u8],
    //     init_args: Option<Vec<u8>>,
    //     nonce: u64,
    //     wallet_index: u64,
    //     gas_limit: u64,
    //     gas_price: u64,
    // ) -> Result<(), Error> {
    //     let hash = bytecode_hash(bytecode.as_slice());
    //     wallet.moonlight_execute(
    //         TransactionData::Deploy(ContractDeploy {
    //             bytecode: ContractBytecode {
    //                 hash,
    //                 bytes: bytecode.clone(),
    //             },
    //             owner: owner.to_vec(),
    //             init_args,
    //             nonce,
    //         }),
    //         wallet_index,
    //         gas_limit,
    //         gas_price,
    //     )?;
    //
    //     Ok(())
    // }

    pub fn call_via_phoenix(
        wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
        contract_id: &ContractId,
        method: impl AsRef<str>,
        args: Vec<u8>,
        wallet_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<(), Error> {
        let mut rng = StdRng::seed_from_u64(0xcafe);
        wallet.phoenix_execute(
            &mut rng,
            TransactionData::Call(ContractCall {
                contract: (*contract_id).into(),
                fn_name: method.as_ref().to_string().clone(),
                fn_args: args,
            }),
            wallet_index,
            gas_limit,
            gas_price,
            0u64,
        )?;

        Ok(())
    }

    pub fn call_via_moonlight(
        wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
        contract_id: &ContractId,
        method: impl AsRef<str>,
        args: Vec<u8>,
        wallet_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<(), Error> {
        wallet.moonlight_execute(
            TransactionData::Call(ContractCall {
                contract: (*contract_id).into(),
                fn_name: method.as_ref().to_string().clone(),
                fn_args: args,
            }),
            wallet_index,
            gas_limit,
            gas_price,
        )?;

        Ok(())
    }
}
