// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::dcli_store::DCliStore;
use execution_core::bytecode::Bytecode;
use execution_core::transfer::{ContractCall, ContractDeploy, ContractExec};
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
    pub fn deploy(
        // todo: rename to deploy_via_moonlight
        wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
        bytecode: &Vec<u8>,
        owner: &Vec<u8>,
        constructor_args: Option<Vec<u8>>,
        nonce: u64,
        wallet_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<(), Error> {
        let mut rng = StdRng::seed_from_u64(0xcafe);
        let hash = bytecode_hash(bytecode.as_slice());
        wallet.phoenix_execute(
            &mut rng,
            ContractExec::Deploy(ContractDeploy {
                bytecode: Bytecode {
                    hash,
                    bytes: bytecode.clone(),
                },
                owner: owner.clone(),
                constructor_args,
                nonce,
            }),
            wallet_index,
            gas_limit,
            gas_price,
            0u64,
        )?;

        Ok(())
    }

    pub fn deploy_via_moonlight(
        wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
        bytecode: &Vec<u8>,
        owner: &Vec<u8>,
        constructor_args: Option<Vec<u8>>,
        nonce: u64,
        wallet_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<(), Error> {
        let hash = bytecode_hash(bytecode.as_slice());
        wallet.moonlight_execute(
            ContractExec::Deploy(ContractDeploy {
                bytecode: Bytecode {
                    hash,
                    bytes: bytecode.clone(),
                },
                owner: owner.clone(),
                constructor_args,
                nonce,
            }),
            wallet_index,
            gas_limit,
            gas_price,
        )?;

        Ok(())
    }

    pub fn call_method(
        // todo: rename to call_via_phoenix
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
            ContractExec::Call(ContractCall {
                contract: contract_id.clone(),
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
            ContractExec::Call(ContractCall {
                contract: contract_id.clone(),
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
