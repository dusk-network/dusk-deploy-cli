// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::borrow::Cow;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Deploy
    #[error("Deploy: {0:?}")]
    #[allow(dead_code)]
    Deploy(Cow<'static, str>),
    /// Wallet
    #[error(transparent)]
    #[allow(dead_code)]
    Wallet(Arc<dusk_wallet::Error>),
    /// Not found error
    #[error("Not found: {0:?}")]
    NotFound(Cow<'static, str>),
    /// IO
    #[error(transparent)]
    IO(Arc<std::io::Error>),
}

impl From<dusk_wallet::Error> for Error {
    fn from(e: dusk_wallet::Error) -> Self {
        Error::Wallet(Arc::from(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(Arc::from(e))
    }
}
