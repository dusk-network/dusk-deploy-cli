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
mod error;
mod executor;
mod gen_id;
mod wallet_builder;

use crate::args::Args;
use crate::block::Block;
use crate::config::BlockchainAccessConfig;
use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::dcli_store::DCliStore;
use crate::error::Error;
use bip39::{Language, Mnemonic, Seed};
use clap::Parser;
use rusk_http_client::{BlockchainInquirer, ContractId, ContractInquirer, RuskHttpClient};
use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::thread;
use tokio::task::JoinSet;
use toml_base_config::BaseConfig;
use tracing::info;
use wallet::Wallet;

use crate::executor::Executor;
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
    let _nonce = cli.nonce;
    let args = cli.args;
    let mut start_bh = cli.block_height;
    let rel_bh = cli.relative_height;
    let _method = cli.method;

    let blockchain_access_config = BlockchainAccessConfig::load_path(config_path)?;

    let mut bytecode_file = File::open(contract_path)?;
    let mut bytecode = Vec::new();
    bytecode_file.read_to_end(&mut bytecode)?;

    let mut _constructor_args: Option<Vec<u8>> = None;
    if !args.is_empty() {
        let v = hex::decode(args).expect("decoding constructore arguments should succeed");
        _constructor_args = Some(v);
    }

    let _wallet_index = 0;

    let phrase = seed_phrase.to_string();
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)
        .map_err(|_| Error::InvalidMnemonicPhrase)?;
    let seed_obj = Seed::new(&mnemonic, "");
    let mut seed = [0u8; 64];
    seed.copy_from_slice(seed_obj.as_bytes());

    let _owner = hex::decode(owner).expect("decoding owner should succeed");

    if rel_bh != 0 {
        let client = RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        if let Ok(cur_bh) = BlockchainInquirer::block_height(&client).wait() {
            start_bh = cur_bh - min(cur_bh, rel_bh);
        }
    }

    let mut join_set = JoinSet::new();

    for index in 0..8 {
        let bytecode = bytecode.clone();
        let wallet = WalletBuilder::build(
            blockchain_access_config.rusk_address.clone(),
            blockchain_access_config.clone().prover_address,
            &seed,
            start_bh,
        )?;
        let blockchain_access_config = blockchain_access_config.clone();
        join_set.spawn(async move {
            do_run(
                index * 250,
                index * 250 + 250,
                index as u64,
                &bytecode,
                &wallet,
                &blockchain_access_config,
                gas_limit,
                gas_price,
            )
        });
        thread::sleep(std::time::Duration::from_millis(2500))
    }

    while let Some(res) = join_set.join_next().wait() {
        println!("res={:?}", res)
    }

    Ok(())
}

fn do_run(
    r1: usize,
    r2: usize,
    wallet_index: u64,
    bytecode: &Vec<u8>,
    wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
    blockchain_access_config: &BlockchainAccessConfig,
    gas_limit: u64,
    gas_price: u64,
) {
    let nonce = 0u64;
    let owner = hex::decode("".to_string()).expect("decoding owner should succeed");
    let method = "value";

    for i in r1..r2 {
        let mut v = Vec::new();
        v.push((i % 256) as u8);
        let constructor_args = Some(v);

        info!("Deploying with nonce {}", nonce + i as u64);
        let result = Executor::deploy_via_moonlight(
            &wallet,
            &bytecode.clone(),
            &owner.clone(),
            constructor_args,
            nonce + i as u64,
            wallet_index,
            gas_limit,
            gas_price,
        );

        match result {
            Ok(_) => info!("Deployment successful {}", i),
            Err(ref err) => info!("{} when deploying", err),
        }

        if result.is_ok() {
            let deployed_id = gen_contract_id(bytecode.clone(), nonce + i as u64, owner.clone());
            info!("Deployed contract id: {}", hex::encode(&deployed_id));

        // println!("verification {}", i);
        // thread::sleep(std::time::Duration::from_secs(15));

        // if !method.is_empty() {
        //     verify_deployment(
        //         &wallet,
        //         deployed_id,
        //         blockchain_access_config.rusk_address.clone(),
        //         method,
        //         wallet_index,
        //         gas_limit,
        //         gas_price,
        //     )
        //     .wait();
        // }
        } else {
            break;
        }
    }
}

// converts seed phrase into a binary seed
#[allow(dead_code)]
fn seed_from_phrase(phrase: impl AsRef<str>) -> Result<[u8; 64], Error> {
    let mnemonic = Mnemonic::from_phrase(phrase.as_ref(), Language::English)
        .map_err(|_| Error::InvalidMnemonicPhrase)?;
    let seed_obj = Seed::new(&mnemonic, "");
    let mut seed = [0u8; 64];
    seed.copy_from_slice(seed_obj.as_bytes());
    Ok(seed)
}

async fn verify_deployment(
    wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
    contract_id: [u8; 32],
    rusk_url: impl AsRef<str>,
    method: impl AsRef<str>,
    wallet_index: u64,
    gas_limit: u64,
    gas_price: u64,
) {
    let random_arg = contract_id[0];
    println!(
        "verifying deployment by calling init({}) and value",
        random_arg
    );
    let method_args = vec![random_arg];

    let r = Executor::call_via_moonlight(
        &wallet,
        &ContractId::from(contract_id),
        "init",
        method_args,
        wallet_index,
        gas_limit,
        gas_price,
    );
    assert!(r.is_ok(), "moonlight call failed");

    let client = RuskHttpClient::new(rusk_url.as_ref().to_string());
    let r = ContractInquirer::query_contract::<(), u8>(
        &client,
        (),
        ContractId::from(contract_id),
        method,
    )
    .await;

    println!("result of calling value: {:?}", r);

    assert_eq!(r.unwrap(), random_arg);
}
