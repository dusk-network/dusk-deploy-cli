// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod args;
mod block;
mod bs58_util;
mod config;
mod contract_deployer;
mod contract_types;
mod dcli_prover_client;
mod dcli_state_client;
mod dcli_store;
mod error;
mod executor;
mod gen_id;
mod seed_util;
mod ser_util;
mod wallet_builder;

use crate::args::Args;
use crate::block::Block;
use crate::bs58_util::Bs58Util;
use crate::config::DDCliConfig;
use crate::contract_deployer::ContractDeployer;
use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::dcli_store::DCliStore;
use crate::error::Error;
use crate::Error::Deploy;
use clap::Parser;
use dusk_bytes::Serializable;
use piecrust_uplink::ContractId;
use rusk_http_client::{BlockchainInquirer, ContractInquirer, RuskHttpClient};
use std::cmp::min;
use std::fs;
use std::fs::File;
use std::io::Read;
use tracing::info;
use wallet::Wallet;

use crate::gen_id::gen_contract_id;
use crate::seed_util::SeedUtil;
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
    let args = cli.args;
    let wallet_index = cli.wallet_index;
    let nonce = cli.nonce;
    let mut start_bh = cli.block_height;
    let rel_bh = cli.relative_height;
    let moonlight_sk_bs58 = cli.moonlight;
    let moonlight: bool = !moonlight_sk_bs58.is_empty();

    let config_content = fs::read_to_string(config_path)?;
    let dd_cli_config = toml::from_str::<DDCliConfig>(config_content.as_str())?;

    let mut bytecode_file = File::open(contract_path)?;
    let mut bytecode = Vec::new();
    bytecode_file.read_to_end(&mut bytecode)?;

    let mut constructor_args: Vec<u8> = Vec::new();
    if !args.is_empty() {
        constructor_args =
            hex::decode(args).expect("decoding constructor arguments should succeed");
    };

    let seed = if moonlight {
        Bs58Util::to_seed(moonlight_sk_bs58)?
    } else {
        SeedUtil::seed_from_phrase(seed_phrase)?
    };

    if !moonlight && rel_bh != 0 {
        let client = RuskHttpClient::new(dd_cli_config.blockchain_access.rusk_address.clone());
        if let Ok(cur_bh) = BlockchainInquirer::block_height(&client).wait() {
            start_bh = cur_bh - min(cur_bh, rel_bh);
        }
    }

    let client = RuskHttpClient::new(dd_cli_config.blockchain_access.rusk_address.clone());
    let wallet = WalletBuilder::build(
        dd_cli_config.blockchain_access.rusk_address.clone(),
        dd_cli_config.blockchain_access.prover_address.clone(),
        &seed,
        start_bh,
    )?;
    println!(
        "wallet2 created, start bh={} seed={}",
        start_bh,
        hex::encode(seed)
    );

    let owner_pk = wallet.account_public_key(0)?;
    let owner_bytes = owner_pk.to_bytes().to_vec();
    let _contract_id = deploy(
        wallet_index,
        &wallet,
        &bytecode,
        &owner_bytes,
        nonce,
        gas_limit,
        gas_price,
        &client,
        &constructor_args,
    )
    .wait()?;

    Ok(())
}

async fn deploy(
    wallet_index: u64,
    wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
    bytecode: &[u8],
    owner: &[u8],
    nonce: u64,
    gas_limit: u64,
    gas_price: u64,
    client: &RuskHttpClient,
    deploy_data: &[u8],
) -> Result<ContractId, Error> {
    let contract_id = ContractDeployer::get_contract_id(&bytecode, nonce, owner);

    let r = ContractInquirer::query_contract::<Vec<u8>, u64>(
        &client,
        owner.to_vec(),
        contract_id,
        "get_balance",
    )
    .await;

    let mut already_exists = false;
    if let Ok(_) = r {
        info!(
            "Contract already exists: {}",
            hex::encode(contract_id.as_bytes())
        );
        already_exists = true;
    }

    if !already_exists {
        info!("Deploying with nonce {}", nonce as u64);
        let result = ContractDeployer::deploy(
            &wallet,
            &bytecode,
            &owner,
            wallet_index,
            nonce,
            gas_limit,
            gas_price,
            deploy_data,
        );

        let _result = match result {
            Ok((existed, Some(contract_id))) => {
                if existed {
                    info!(
                        "Contract already exists: {}",
                        hex::encode(contract_id.as_bytes())
                    );
                } else {
                    info!(
                        "Deployment successful: {}",
                        hex::encode(contract_id.as_bytes())
                    );
                }
                Ok(contract_id)
            }
            Ok((_, None)) => Err(Deploy("Could not determine contract id".into())),
            Err(err) => Err(err),
        };
    }
    Ok(contract_id)
}
