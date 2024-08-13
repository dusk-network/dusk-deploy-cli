// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Error, QueryResult, RuskHttpClient, RuskRequest, SpentTxResponse};
use std::borrow::Cow;

pub struct BlockchainInquirer;

impl BlockchainInquirer {
    pub async fn retrieve_tx_err<S>(
        txid: S,
        client: &RuskHttpClient,
    ) -> Result<Option<String>, Error>
    where
        S: AsRef<str>,
    {
        let query = "query { tx(hash: \"####\") { id, err }}".replace("####", txid.as_ref());
        let response = Self::gql_query(client, query.as_str()).await?;
        let result = serde_json::from_slice::<SpentTxResponse>(&response)?;
        match result.tx {
            Some(tx) => Ok(tx.err),
            None => Err(Error::NotFound(Cow::from(txid.as_ref().to_string()))),
        }
    }

    pub async fn block_height(client: &RuskHttpClient) -> Result<u64, Error> {
        let query = "query { block(height: -1) {header { height}} }";
        let response = Self::gql_query(client, query).await?;
        let result = serde_json::from_slice::<QueryResult>(&response)?;
        Ok(result.block.header.height)
    }

    pub async fn gql_query(
        client: &RuskHttpClient,
        query: impl AsRef<str>,
    ) -> Result<Vec<u8>, Error> {
        let request = RuskRequest::new("gql", query.as_ref().as_bytes().to_vec());
        client.call(2, "Chain", &request).await
    }
}
