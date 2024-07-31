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
mod deployer;
mod error;
mod wallet_builder;

use crate::args::Args;
use crate::config::BlockchainAccessConfig;
use crate::error::Error;
use clap::Parser;
use std::fs::File;
use std::io::Read;
use toml_base_config::BaseConfig;

use crate::deployer::Deployer;

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

    // let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    // let _ = fs::metadata(config_path)
    //     .map_err(|_| Error::NotFound(config_path.to_string_lossy().into_owned().into()))?;
    let blockchain_access_config = BlockchainAccessConfig::load_path(config_path)?;
    // let psw = if pwd_hash.is_empty() {
    //     Pwd(password)
    // } else {
    //     PwdHash(pwd_hash)
    // };

    let mut bytecode_file = File::open(contract_path)?;
    let mut bytecode = Vec::new();
    bytecode_file.read_to_end(&mut bytecode)?;

    // let wallet_accessor = WalletAccessor::create(wallet_path.clone(), psw.clone())?;
    // let wallet = Wallet::from_file(wallet_accessor)?;

    // let (_psk, _ssk) = wallet.spending_keys(wallet.default_address())?;

    let constructor_args: Option<Vec<u8>> = None;

    let wallet_index = 0;

    let result = Deployer::deploy(
        blockchain_access_config.rusk_address,
        blockchain_access_config.prover_address,
        bytecode,
        owner,
        constructor_args,
        nonce,
        wallet_index,
        gas_limit,
        gas_price,
    );

    println!("deployment result = {:?}", result);

    Ok(())
}
