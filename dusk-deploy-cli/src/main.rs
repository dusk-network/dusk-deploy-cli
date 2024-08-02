// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod args;
mod block;
mod config;
mod dcli_prover_client;
mod dcli_state_client;
mod dcli_store;
mod deployer;
mod error;
mod gen_id;
mod wallet_builder;

use crate::args::Args;
use crate::config::BlockchainAccessConfig;
use crate::error::Error;
use clap::Parser;
use execution_core::transfer::ContractId;
use rusk_http_client::{ContractInquirer, RuskHttpClient};
use std::fs::File;
use std::io::Read;
use std::thread;
use toml_base_config::BaseConfig;

use crate::deployer::Deployer;
use crate::gen_id::gen_contract_id;

#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Error> {
    let cli = Args::parse();

    let config_path = cli.config_path.as_path();
    let _wallet_path = cli.wallet_path.as_path();
    let _password = cli.wallet_pass;
    let _pwd_hash = cli.pwd_hash;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;
    let contract_path = cli.contract_path.as_path();
    let owner = cli.owner;
    let nonce = cli.nonce;

    let blockchain_access_config = BlockchainAccessConfig::load_path(config_path)?;

    let mut bytecode_file = File::open(contract_path)?;
    let mut bytecode = Vec::new();
    bytecode_file.read_to_end(&mut bytecode)?;

    let constructor_args: Option<Vec<u8>> = None;

    let wallet_index = 0;

    let seed_vec = hex::decode("7965013909185294fa0f0d2a2be850ee89389e45d17e0c7da9a7588901648086c5b3ac52d95b6fd421104b6a77ca21772f0a041f031c3c8039ae3b24c48467bd")
        .expect("decoding seed should succeed");
    let mut seed = [0u8; 64];
    seed.copy_from_slice(seed_vec.as_slice());

    let owner = hex::decode(owner).expect("decoding owner should succeed");

    let result = Deployer::deploy(
        blockchain_access_config.rusk_address.clone(),
        blockchain_access_config.prover_address,
        &bytecode,
        &owner,
        constructor_args,
        nonce,
        wallet_index,
        gas_limit,
        gas_price,
        &seed,
    );

    println!("deployment result = {:?}", result);
    let deployed_id = gen_contract_id(bytecode, nonce, owner);
    println!("deployed contract id = {}", hex::encode(&deployed_id));

    thread::sleep(std::time::Duration::from_secs(15));

    let client = RuskHttpClient::new(blockchain_access_config.rusk_address);
    let r = ContractInquirer::query_contract::<(), ()>(
        &client,
        (),
        ContractId::from(deployed_id),
        "ping",
    )
    .await;
    println!("result of calling the contract's method: {:?}", r);

    Ok(())
}
