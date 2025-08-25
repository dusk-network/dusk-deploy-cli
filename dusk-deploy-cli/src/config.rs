// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct DDCliConfig {
    pub blockchain_access: BlockchainAccess,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct BlockchainAccess {
    pub rusk_address: String,
    pub prover_address: String,
}
