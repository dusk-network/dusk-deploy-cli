// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::dcli_prover_client::DCliProverClient;
use crate::dcli_state_client::DCliStateClient;
use crate::wallet_builder::DCliStore;
use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Deploy
    #[error("Deploy: {0:?}")]
    #[allow(dead_code)]
    Deploy(Cow<'static, str>),
    // Wallet
    // #[error(transparent)]
    // #[allow(dead_code)]
    // Wallet(Arc<dusk_wallet::Error>),
    // Not found error
    // #[error("Not found: {0:?}")]
    // NotFound(Cow<'static, str>),
    /// IO
    #[error(transparent)]
    IO(Arc<std::io::Error>),
    /// Rkyv errors
    #[error("Serialization error")]
    Rkyv,
    /// Http client errors
    #[error(transparent)]
    HttpClient(Arc<reqwest::Error>),
    /// Bytes Serialization Errors
    #[error("Serialization error occurred: {0:?}")]
    Serialization(Arc<dusk_bytes::Error>),
    /// Prover Errors
    #[error("Prover error occurred: {0:?}")]
    Prover(Arc<rusk_prover::ProverError>),
    /// Moat Errors
    #[error("Prover error occurred: {0:?}")]
    Moat(Arc<zk_citadel_moat::Error>),
    /// Wallet Errors
    #[error("Wallet error occurred: {0:?}")]
    Wallet(Arc<dusk_wallet::Error>),
    /// Wallet2 Errors
    #[error("Wallet2 error occurred")] // todo
    Wallet2(Arc<wallet::Error<DCliStore, DCliStateClient, DCliProverClient>>), // todo: rename
}

impl From<wallet::Error<DCliStore, DCliStateClient, DCliProverClient>> for Error {
    fn from(e: wallet::Error<DCliStore, DCliStateClient, DCliProverClient>) -> Self {
        Error::Wallet2(Arc::from(e))
    }
}

impl From<dusk_wallet::Error> for Error {
    fn from(e: dusk_wallet::Error) -> Self {
        Error::Wallet(Arc::from(e))
    }
}

impl From<zk_citadel_moat::Error> for Error {
    fn from(e: zk_citadel_moat::Error) -> Self {
        Error::Moat(Arc::from(e))
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

impl From<rusk_prover::ProverError> for Error {
    fn from(err: rusk_prover::ProverError) -> Self {
        Error::Prover(Arc::from(err))
    }
}
