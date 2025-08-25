// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use bytecheck::CheckBytes;
use rkyv::ser::serializers::{BufferScratch, BufferSerializer, CompositeSerializer};
use rkyv::ser::Serializer;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Infallible, Serialize};

use piecrust_uplink::StandardBufSerializer;

pub const SCRATCH_BUF_BYTES: usize = 1024;
pub const RKYV_BUF_SIZE: usize = 0x10000;

pub struct SerUtil;

impl SerUtil {
    pub fn serialize_init_argument<A>(arg: &A) -> Vec<u8>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        let mut sbuf = [0u8; SCRATCH_BUF_BYTES];
        let mut buffer = [0u8; RKYV_BUF_SIZE];
        let scratch = BufferScratch::new(&mut sbuf);
        let ser = BufferSerializer::new(&mut buffer[..]);
        let mut ser = CompositeSerializer::new(ser, scratch, Infallible);
        ser.serialize_value(arg)
            .expect("failed to rkyv serialize arg");
        let pos = ser.pos();
        buffer[..pos].to_vec()
    }
}
