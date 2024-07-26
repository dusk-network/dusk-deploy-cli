// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod args;
mod contract_caller;
mod error;

use crate::error::Error;
use crate::args::Args;
use std::fs;
use clap::Parser;
use toml_base_config::BaseConfig;

use dusk_wallet::{Wallet, WalletPath};
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};
use wallet_accessor::Password::{Pwd, PwdHash};


#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Error> {
    let cli = Args::parse();

    let config_path = cli.config_path.as_path();
    let wallet_path = cli.wallet_path.as_path();
    let password = cli.wallet_pass;
    let pwd_hash = cli.pwd_hash;
    let gas_limit = cli.gas_limit;
    let gas_price = cli.gas_price;
    let contract_path = cli.contract_path.as_path();
    let owner = cli.owner;
    let nonce = cli.nonce;

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let _ = fs::metadata(config_path).map_err(|_| {
        Error::NotFound(config_path.to_string_lossy().into_owned().into())
    })?;
    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;
    let psw = if pwd_hash.is_empty() {
        Pwd(password)
    } else {
        PwdHash(pwd_hash)
    };

    let wallet_accessor =
        WalletAccessor::create(wallet_path.clone(), psw.clone())?;
    let wallet = Wallet::from_file(wallet_accessor)?;

    let (_psk, ssk) = wallet.spending_keys(wallet.default_address())?;

    println!("hello dusk-deploy-cli");

    Ok(())
}
