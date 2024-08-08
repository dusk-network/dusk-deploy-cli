// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::bc_types::MAX_CALL_SIZE;
use crate::block::BlockInPlace;
use crate::error::Error;
use crate::Error::InvalidQueryResponse;
use crate::{RuskHttpClient, RuskRequest};
use bytecheck::CheckBytes;
use bytes::Bytes;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{check_archived_root, Archive, Deserialize, Infallible};

pub type ContractId = [u8; 32];

pub struct ContractInquirer {}

impl ContractInquirer {
    /// Calls a given query method of a given contract.
    pub async fn query_contract<A, R>(
        client: &RuskHttpClient,
        args: A,
        contract_id: ContractId,
        method: impl AsRef<str>,
    ) -> Result<R, Error>
    where
        A: Archive,
        A: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<MAX_CALL_SIZE>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible> + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let contract_id = hex::encode(contract_id.as_slice());
        let response = client
            .contract_query::<A, MAX_CALL_SIZE>(contract_id.as_ref(), method.as_ref(), &args)
            .await?;

        let response_data = check_archived_root::<R>(response.as_slice())
            .map_err(|_| InvalidQueryResponse("rkyv deserialization error".into()))?;
        let r = response_data
            .deserialize(&mut Infallible)
            .expect("Infallible");
        Ok(r)
    }

    /// Calls a given query method of a given contract.
    /// Returns response as a stream to be processed by the caller.
    pub async fn query_contract_with_feeder<A>(
        client: &RuskHttpClient,
        args: A,
        contract_id: ContractId,
        method: impl AsRef<str>,
    ) -> Result<impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>, Error>
    where
        A: Archive,
        A: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<MAX_CALL_SIZE>>,
    {
        let contract_id = hex::encode(contract_id.as_slice());
        let req = rkyv::to_bytes(&args)
            .expect("Serializing should be infallible")
            .to_vec();
        let stream = client
            .call_raw(
                1,
                contract_id.as_ref(),
                &RuskRequest::new(method.as_ref(), req),
                true,
            )
            .wait()?
            .bytes_stream();
        Ok(stream)
    }
}
