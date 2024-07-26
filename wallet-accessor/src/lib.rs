// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

pub mod accessor;
mod config;

pub use accessor::{Password, WalletAccessor, MAX_CALL_SIZE};
pub use config::BlockchainAccessConfig;
