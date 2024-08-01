// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::borrow::Cow;
use std::io;

/// Errors returned by this library
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Rkyv errors
    #[error("A serialization error occurred.")]
    Rkyv,
    /// Reqwest errors
    #[error("A request error occurred: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Filesystem errors
    #[error(transparent)]
    IO(#[from] io::Error),
    /// Rusk error
    #[error("Rusk error occurred: {0}")]
    Rusk(String),
    /// Query error
    #[error("Invalid query response: {0:?}")]
    InvalidQueryResponse(Cow<'static, str>),
    /// Stream error
    #[error("Stream item not present or stream error: {0:?}")]
    Stream(Cow<'static, str>),
}
