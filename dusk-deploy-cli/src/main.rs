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
use bip39::{Language, Mnemonic, Seed};
use clap::Parser;
use execution_core::transfer::ContractId;
use rusk_http_client::{ContractInquirer, RuskHttpClient};
use std::fs::File;
use std::io::Read;
use toml_base_config::BaseConfig;
use tracing::info;

use crate::deployer::Deployer;
use crate::gen_id::gen_contract_id;
use crate::wallet_builder::WalletBuilder;

#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(|_| Error::Tracing)?;

    let cli = Args::parse();

    let config_path = cli.config_path.as_path();
    let seed_phrase = cli.seed;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;
    let contract_path = cli.contract_path.as_path();
    let owner = cli.owner;
    let nonce = cli.nonce;
    let _args = cli.args;
    let start_block_height = cli.block_height;
    let method = cli.method;

    let blockchain_access_config = BlockchainAccessConfig::load_path(config_path)?;

    let mut bytecode_file = File::open(contract_path)?;
    let mut bytecode = Vec::new();
    bytecode_file.read_to_end(&mut bytecode)?;

    // let mut constructor_args: Option<Vec<u8>> = None;
    // if !args.is_empty() {
    //     let v = hex::decode(args).expect("decoding constructor arguments should succeed");
    //     constructor_args = Some(v);
    // }

    let wallet_index = 0;

    let phrase = seed_phrase.to_string();
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)
        .map_err(|_| Error::InvalidMnemonicPhrase)?;
    let seed_obj = Seed::new(&mnemonic, "");
    let mut seed = [0u8; 64];
    seed.copy_from_slice(seed_obj.as_bytes());

    let owner = hex::decode(owner).expect("decoding owner should succeed");

    let wallet = WalletBuilder::build(
        blockchain_access_config.rusk_address.clone(),
        blockchain_access_config.clone().prover_address,
        &seed,
        start_block_height,
    )?;

    for i in 683..768 {
        let mut v = Vec::new();
        v.push((i % 256) as u8);
        let constructor_args = Some(v);

        let result = Deployer::deploy(
            &wallet,
            &bytecode.clone(),
            &owner.clone(),
            constructor_args,
            nonce + i,
            wallet_index,
            gas_limit,
            gas_price,
        );

        match result {
            Ok(_) => info!("Deployment successful {}", i),
            Err(ref err) => info!("{} for {:?}", err, contract_path),
        }

        if result.is_ok() {
            let deployed_id = gen_contract_id(bytecode.clone(), nonce + i, owner.clone());
            info!("Deployed contract id: {}", hex::encode(&deployed_id));

            if !method.clone().is_empty() {
                verify_deployment(
                    deployed_id,
                    blockchain_access_config.rusk_address.clone(),
                    method.clone(),
                )
                .await;
            }
        } else {
            break;
        }
    }

    Ok(())
}

async fn verify_deployment(
    contract_id: [u8; 32],
    rusk_url: impl AsRef<str>,
    method: impl AsRef<str>,
) {
    println!(
        "verifying deployment by calling contract's method: {}",
        method.as_ref(),
    );

    let client = RuskHttpClient::new(rusk_url.as_ref().to_string());
    let r = ContractInquirer::query_contract::<(), u8>(
        &client,
        (),
        ContractId::from(contract_id),
        method,
    )
    .await;

    println!("result of calling the contract's method: {:x?}", r);
}
