// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

pub const MAX_CALL_SIZE: usize = 65536;
pub const MAX_RESPONSE_SIZE: usize = 65536;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTx {
    pub id: String,
    #[serde(default)]
    pub raw: String,
    pub err: Option<String>,
    #[serde(alias = "gasSpent", default)]
    pub gas_spent: f64,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct SpentTxResponse {
    pub tx: Option<SpentTx>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Header {
    pub height: u64,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Block {
    pub header: Header,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct QueryResult {
    pub block: Block,
}
