// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::dcli_store::DCliStore;
use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Deploy
    #[error("{0}")]
    Deploy(Cow<'static, str>),
    /// Propagate
    #[error("{0}")]
    Propagate(Cow<'static, str>),
    /// IO
    #[error(transparent)]
    IO(Arc<std::io::Error>),
    /// Rkyv errors
    #[error("Serialization error")]
    Rkyv,
    /// Http client errors
    #[error(transparent)]
    HttpClient(Arc<reqwest::Error>),
    /// Rusk http client errors
    #[error(transparent)]
    RuskHttpClient(Arc<rusk_http_client::Error>),
    /// Bytes Serialization Errors
    #[error("Serialization error occurred: {0:?}")]
    Serialization(Arc<dusk_bytes::Error>),
    /// Wallet Errors
    #[error("Wallet error occurred: {0}")]
    Wallet(Arc<wallet::Error<DCliStore, DCliStateClient, DCliProverClient>>),
    /// Seed phrase is not valid
    #[error("Invalid recovery phrase")]
    InvalidMnemonicPhrase,
    /// Tracing errors
    #[error("Tracing error")]
    Tracing,
    /// Base 58 errors
    #[error(transparent)]
    Base58(Arc<bs58::decode::Error>),
}

impl From<wallet::Error<DCliStore, DCliStateClient, DCliProverClient>> for Error {
    fn from(e: wallet::Error<DCliStore, DCliStateClient, DCliProverClient>) -> Self {
        Error::Wallet(Arc::from(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(Arc::from(e))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpClient(Arc::from(e))
    }
}

impl From<dusk_bytes::Error> for Error {
    fn from(err: dusk_bytes::Error) -> Self {
        Self::Serialization(Arc::from(err))
    }
}

impl From<rusk_http_client::Error> for Error {
    fn from(err: rusk_http_client::Error) -> Self {
        Error::RuskHttpClient(Arc::from(err))
    }
}

impl From<bs58::decode::Error> for Error {
    fn from(err: bs58::decode::Error) -> Self {
        Error::Base58(Arc::from(err))
    }
}
