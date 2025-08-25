// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

use dusk_core::signatures::bls::PublicKey as BlsAddress;
use piecrust_uplink::ContractId;

/// The `DuskDS` address. This can be either a public account or a contract-id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub enum Address {
    /// An externally owned public-key.
    External(BlsAddress),
    /// A contract-id.
    Contract(ContractId),
}
