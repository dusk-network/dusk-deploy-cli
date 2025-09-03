// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use piecrust_uplink::ContractId;
use std::io::{self, Write};

use crate::error::Error;
use reqwest::{Body, Response};
use rkyv::Archive;

/// Supported Rusk version
const REQUIRED_RUSK_VERSION: &str = "1.0.0-rc.0";

/// Target for contracts
pub const CONTRACTS_TARGET: &str = "contracts";

#[derive(Debug)]
/// RuskRequesst according to the rusk event system
pub struct RuskRequest {
    topic: String,
    data: Vec<u8>,
}

impl RuskRequest {
    /// New RuskRequesst from topic and data
    pub fn new(topic: &str, data: Vec<u8>) -> Self {
        let topic = topic.to_string();
        Self { data, topic }
    }

    /// Return the binary representation of the RuskRequesst
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buffer = vec![];
        buffer.write_all(&(self.topic.len() as u32).to_le_bytes())?;
        buffer.write_all(self.topic.as_bytes())?;
        buffer.write_all(&self.data)?;

        Ok(buffer)
    }
}
#[derive(Clone)]
/// Rusk HTTP Binary Client
pub struct RuskHttpClient {
    uri: String,
}

impl RuskHttpClient {
    /// Create a new HTTP Client
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    /// Utility for querying the rusk VM
    pub async fn contract_query<I, const N: usize>(
        &self,
        contract: impl AsRef<str>,
        method: impl AsRef<str>,
        value: &I,
    ) -> Result<Vec<u8>, Error>
    where
        I: Archive,
        I: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<N>>,
    {
        let data = rkyv::to_bytes(value).map_err(|_| Error::Rkyv)?.to_vec();

        let response = self
            .call_raw(CONTRACTS_TARGET, contract, method, &data, false, false)
            .await?;

        Ok(response.bytes().await?.to_vec())
    }

    /// Utility for querying the rusk VM with JSON
    pub async fn contract_query_json(
        &self,
        contract: impl AsRef<str>,
        method: impl AsRef<str>,
        json_value: impl AsRef<str>,
    ) -> Result<Vec<u8>, Error> {
        let data = json_value.as_ref().as_bytes();
        let response = self
            .call_raw(CONTRACTS_TARGET, contract, method, &data, false, true)
            .await?;

        Ok(response.bytes().await?.to_vec())
    }

    /// Check rusk connection
    pub async fn check_connection(&self) -> Result<(), reqwest::Error> {
        reqwest::Client::new().post(&self.uri).send().await?;
        Ok(())
    }

    /// Send a RuskRequest to a specific target.
    ///
    /// The response is interpreted as Binary
    pub async fn call(
        &self,
        target: &str,
        entity: impl AsRef<str>,
        topic: &str,
        request_data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let response = self
            .call_raw(target, entity, topic, request_data, false, false)
            .await?;
        let data = response.bytes().await?;
        Ok(data.to_vec())
    }

    /// Send a RuskRequest to a specific target without parsing the response
    pub async fn call_raw(
        &self,
        target: impl AsRef<str>,
        entity: impl AsRef<str>,
        topic: impl AsRef<str>,
        request_data: &[u8],
        feed: bool,
        is_json: bool,
    ) -> Result<Response, Error> {
        let uri = &self.uri;
        let client = reqwest::Client::new();
        let target = target.as_ref();
        let topic = topic.as_ref();
        let entity = if entity.as_ref().is_empty() {
            entity.as_ref().to_string()
        } else {
            format!(":{}", entity.as_ref())
        };
        let rues_prefix = if uri.ends_with('/') { "on" } else { "/on" };
        let content_type = if is_json {
            "application/json"
        } else {
            "application/octet-stream"
        };
        let mut request = client
            .post(format!("{uri}{rues_prefix}/{target}{entity}/{topic}"))
            .body(Body::from(request_data.to_vec()))
            .header("Content-Type", content_type)
            .header("rusk-version", REQUIRED_RUSK_VERSION);

        if feed {
            request = request.header("Rusk-Feeder", "1");
        }
        let response = request.send().await?;

        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let error = &response.bytes().await?;

            let error = String::from_utf8(error.to_vec()).unwrap_or("unparsable error".into());

            let msg = format!("{status}: {error}");

            Err(Error::Rusk(msg))
        } else {
            Ok(response)
        }
    }

    /// Upload the data driver
    pub async fn upload_driver(
        &self,
        driver_bytecode: impl AsRef<[u8]>,
        contract_id: &ContractId,
        hash: impl AsRef<[u8]>,
        signature: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, Error> {
        let uri = &self.uri;
        let client = reqwest::Client::new();
        let target = "upload_driver";
        let entity = hex::encode(contract_id.as_bytes());
        let entity = if entity.is_empty() {
            entity.to_string()
        } else {
            format!(":{}", entity)
        };
        let topic = "aa/bb";
        let rues_prefix = if uri.ends_with('/') { "on" } else { "/on" };
        let request = client
            .post(format!("{uri}{rues_prefix}/{target}{entity}/{topic}"))
            .body(Body::from(driver_bytecode.as_ref().to_vec()))
            .header("Content-Type", "application/octet-stream")
            .header("rusk-version", REQUIRED_RUSK_VERSION)
            .header("hash", hex::encode(hash.as_ref()))
            .header("sign", hex::encode(signature.as_ref()));

        println!("request={:?}", request);

        let response = request.send().await?;

        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let error = &response.bytes().await?;

            let error = String::from_utf8(error.to_vec()).unwrap_or("unparsable error".into());

            let msg = format!("{status}: {error}");

            Err(Error::Rusk(msg))
        } else {
            let data = response.bytes().await?;
            Ok(data.to_vec())
        }
    }

    /// Call data driver
    pub async fn call_driver(
        &self,
        contract_id: &ContractId,
        driver_method: impl AsRef<str>,
        contract_method: impl AsRef<str>,
        data: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, Error> {
        let uri = &self.uri;
        let client = reqwest::Client::new();
        let target = "driver";
        let entity = hex::encode(contract_id.as_bytes());
        let entity = if entity.is_empty() {
            entity.to_string()
        } else {
            format!(":{}", entity)
        };
        // "encode_input_fn:get_version"
        let (driver_method, contract_method) = (driver_method.as_ref(), contract_method.as_ref());
        let topic = format!("{driver_method}:{contract_method}");
        let rues_prefix = if uri.ends_with('/') { "on" } else { "/on" };
        let data = data.as_ref().to_vec();
        let data_len = data.len();
        let request_builder = client
            .post(format!("{uri}{rues_prefix}/{target}{entity}/{topic}"))
            .body(Body::from(data))
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", data_len)
            .header("rusk-version", REQUIRED_RUSK_VERSION);
        let request = request_builder.build()?;

        println!("request={:?}", request);
        println!("request body={:x?}", request.body().unwrap().as_bytes());

        // let response = request.send().await?;
        let response = client.execute(request).await?;

        println!("response={:?}", response);
        let r = response.bytes().await?;
        println!("response bytes={:?}", r);
        Ok(r.to_vec())
    }
}
