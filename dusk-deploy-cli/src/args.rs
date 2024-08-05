// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Args {
    /// Blockchain access config directory
    #[clap(long, default_value = "./config.toml")]
    pub config_path: PathBuf,

    /// Seed phrase [default: ``]
    #[clap(long, default_value_t = String::from("spice property autumn primary undo innocent pole legend stereo mom eternal topic"))]
    pub seed: String,

    /// Gas limit [default: `500000000`]
    #[clap(long, default_value_t = 500000000)]
    pub gas_limit: u64,

    /// Gas price [default: `1`]
    #[clap(long, default_value_t = 1)]
    pub gas_price: u64,

    /// Path to contract code
    #[clap(short, long, default_value = "./test/bob.wasm")]
    pub contract_path: PathBuf,

    /// Hexadecimal string of contract's owner [default: ``]
    #[clap(short, long, default_value_t = String::from(""))]
    pub owner: String,

    /// Nonce [default: `0`]
    #[clap(long, default_value_t = 0)]
    pub nonce: u64,

    /// Hexadecimal string of contract's constructor arguments [default: ``]
    #[clap(short, long, default_value_t = String::from(""))]
    pub args: String,

    /// Method to call [default: ``]
    #[clap(short, long, default_value_t = String::from(""))]
    pub method: String,
}
