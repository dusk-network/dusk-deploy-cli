// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use serde::{Deserialize, Serialize};
use toml_base_config::BaseConfig;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct BlockchainAccessConfig {
    pub rusk_address: String,
    pub prover_address: String,
}

impl BaseConfig for BlockchainAccessConfig {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}
