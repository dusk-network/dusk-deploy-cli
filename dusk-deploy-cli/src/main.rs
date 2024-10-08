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
use crate::error::Error;
use bip39::{Language, Mnemonic, Seed};
use clap::Parser;
use rusk_http_client::{BlockchainInquirer, RuskHttpClient};
use std::cmp::min;
use std::fs::File;
use std::io::Read;
use toml_base_config::BaseConfig;
use tracing::info;

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
    let nonce = cli.nonce;
    let args = cli.args;
    let mut start_bh = cli.block_height;
    let rel_bh = cli.relative_height;
    let moonlight_sk_bs58 = cli.moonlight;
    let moonlight: bool = !moonlight_sk_bs58.is_empty();

    let blockchain_access_config = BlockchainAccessConfig::load_path(config_path)?;

    let mut bytecode_file = File::open(contract_path)?;
    let mut bytecode = Vec::new();
    bytecode_file.read_to_end(&mut bytecode)?;

    let mut constructor_args: Option<Vec<u8>> = None;
    if !args.is_empty() {
        let v = hex::decode(args).expect("decoding constructor arguments should succeed");
        constructor_args = Some(v);
    }

    let wallet_index = 0;

    let seed = if moonlight {
        seed_from_bs58(moonlight_sk_bs58)?
    } else {
        seed_from_phrase(seed_phrase)?
    };

    let owner = hex::decode(owner).expect("decoding owner should succeed");

    if !moonlight && rel_bh != 0 {
        let client = RuskHttpClient::new(blockchain_access_config.rusk_address.clone());
        if let Ok(cur_bh) = BlockchainInquirer::block_height(&client).wait() {
            start_bh = cur_bh - min(cur_bh, rel_bh);
        }
    }

    let wallet = WalletBuilder::build(
        blockchain_access_config.rusk_address.clone(),
        blockchain_access_config.clone().prover_address,
        &seed,
        start_bh,
    )?;

    let result = if moonlight {
        Executor::deploy_via_moonlight(
            &wallet,
            &bytecode,
            &owner,
            constructor_args,
            nonce,
            wallet_index,
            gas_limit,
            gas_price,
        )
    } else {
        Executor::deploy_via_phoenix(
            &wallet,
            &bytecode,
            &owner,
            constructor_args,
            nonce,
            wallet_index,
            gas_limit,
            gas_price,
        )
    };

    match result {
        Ok(_) => info!("Deployment successful"),
        Err(ref err) => info!("{} when deploying {:?}", err, contract_path),
    }

    if result.is_ok() {
        let deployed_id = gen_contract_id(bytecode, nonce, owner);
        info!("Deployed contract id: {}", hex::encode(deployed_id));
    }

    Ok(())
}

// converts seed phrase into a binary seed
fn seed_from_phrase(phrase: impl AsRef<str>) -> Result<[u8; 64], Error> {
    let mnemonic = Mnemonic::from_phrase(phrase.as_ref(), Language::English)
        .map_err(|_| Error::InvalidMnemonicPhrase)?;
    let seed_obj = Seed::new(&mnemonic, "");
    let mut seed = [0u8; 64];
    seed.copy_from_slice(seed_obj.as_bytes());
    Ok(seed)
}

// converts base 58 string into a binary seed
fn seed_from_bs58(bs58_str: impl AsRef<str>) -> Result<[u8; 64], Error> {
    let v = bs58::decode(bs58_str.as_ref()).into_vec()?;
    let mut seed = [0u8; 64];
    seed[0..32].copy_from_slice(&v);
    Ok(seed)
}
