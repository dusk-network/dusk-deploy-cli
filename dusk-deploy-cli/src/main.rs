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
    let wallet_index = cli.wallet_index;
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


    let seed = if moonlight {
        Bs58Util::to_seed(moonlight_sk_bs58)?
    } else {
        SeedUtil::seed_from_phrase(seed_phrase)?
    };

    let owner = hex::decode(owner).expect("decoding owner should succeed");

    if !moonlight && rel_bh != 0 {
        let client = RuskHttpClient::new(dac_cli_config.blockchain_access.rusk_address.clone());
        if let Ok(cur_bh) = BlockchainInquirer::block_height(&client).wait() {
            start_bh = cur_bh - min(cur_bh, rel_bh);
        }
    }

    let wallet = WalletBuilder::build(
        dac_cli_config.blockchain_access.rusk_address.clone(),
        dac_cli_config.blockchain_access.prover_address.clone(),
        &seed,
        start_bh,
    )?;

    let contract_id = deploy(
        wallet_index,
        &wallet,
        &bytecode,
        &driver_bytecode,
        &owner_bytes,
        nonce,
        deploy_gas_limit,
        deploy_gas_price,
        &client,
        &deploy_data,
    )
    .wait()?;
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

async fn deploy(
    wallet_index: u64,
    wallet: &Wallet<DCliStore, DCliStateClient, DCliProverClient>,
    bytecode: &[u8],
    driver_bytecode: &[u8],
    owner: &[u8],
    nonce: u64,
    gas_limit: u64,
    gas_price: u64,
    client: &RuskHttpClient,
    deploy_data: &DacDeployData,
) -> Result<ContractId, Error> {
    let contract_id = DacDeployer::get_contract_id(&bytecode, nonce, owner);

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
        let result = DacDeployer::deploy(
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
