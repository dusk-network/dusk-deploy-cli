// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use execution_core::bytecode::Bytecode;
use execution_core::transfer::{ContractDeploy, ContractExec};
use rand::prelude::*;
use rand::rngs::StdRng;

use crate::wallet_builder::WalletBuilder;
use crate::Error;

fn bytecode_hash(bytecode: impl AsRef<[u8]>) -> [u8; 32] {
    let hash = blake3::hash(bytecode.as_ref());
    hash.into()
}

pub struct Deployer;

impl Deployer {
    pub fn deploy(
        rusk_http_client_url: impl AsRef<str>,
        bytecode: Vec<u8>,
        owner: impl AsRef<[u8]>,
        constructor_args: Option<Vec<u8>>,
        nonce: u64,
        wallet_index: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Result<(), Error> {
        let mut rng = StdRng::seed_from_u64(0xcafe);
        let hash = bytecode_hash(bytecode.as_slice());
        let wallet = WalletBuilder::build(rusk_http_client_url)?;
        wallet
            .phoenix_execute(
                &mut rng,
                ContractExec::Deploy(ContractDeploy {
                    bytecode: Bytecode {
                        hash,
                        bytes: bytecode,
                    },
                    owner: owner.as_ref().to_vec(),
                    constructor_args,
                    nonce,
                }),
                wallet_index,
                gas_limit,
                gas_price,
                0u64,
            )
            .expect("Deployment should succeed"); // todo: error processing

        Ok(())
    }
}
