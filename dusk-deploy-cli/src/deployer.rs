// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_state_client::DCliStateClient;
use execution_core::bytecode::Bytecode;
use execution_core::transfer::{ContractDeploy, ContractExec};
use rand::prelude::*;
use rand::rngs::StdRng;
use wallet::Wallet;

use crate::wallet_builder::{DcliProverClient, DcliStore, WalletBuilder};
use crate::Error;

fn bytecode_hash(bytecode: impl AsRef<[u8]>) -> [u8; 32] {
    let hash = blake3::hash(bytecode.as_ref());
    hash.into()
}

pub struct Deployer;

impl Deployer {
    pub fn deployer() -> Result<(), Error> {
        let mut rng = StdRng::seed_from_u64(0xcafe);

        let constructor_args = Some(vec![init_value]);

        let hash = bytecode_hash(bytecode.as_ref());
        let wallet = WalletBuilder::build()?;
        wallet.phoenix_execute(
            rng,
            ContractExec::Deploy(ContractDeploy {
                bytecode: Bytecode {
                    hash,
                    bytes: bytecode.as_ref().to_vec(),
                },
                owner: OWNER.to_vec(),
                constructor_args,
                nonce: 0,
            }),
            SENDER_INDEX,
            gas_limit,
            GAS_PRICE,
            0u64,
        );

        Ok(())
    }
}
