// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod error;

use crate::error::Error;

#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Error> {
    println!("hello dusk-deploy-cli");
    Ok(())
}
