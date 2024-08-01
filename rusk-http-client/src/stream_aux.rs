// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::block::BlockInPlace;
use crate::Error;
use bytecheck::CheckBytes;
use bytes::Bytes;
use futures_util::StreamExt;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Infallible};

pub struct StreamAux;

impl StreamAux {
    /// Finds and returns items for which
    /// the given filter returns true,
    pub fn find_items<R, const L: usize>(
        mut filter_collect: impl FnMut(&R),
        stream: &mut (impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
                  + std::marker::Unpin),
    ) -> Result<(), Error>
    where
        R: Archive + Clone,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>
            + Deserialize<R, SharedDeserializeMap>,
    {
        let mut remainder = Vec::<u8>::new();
        while let Some(chunk) = stream.next().wait() {
            let mut buffer = vec![];
            buffer.append(&mut remainder);
            buffer.extend_from_slice(&chunk.map_err(|_| Error::Stream("chunking error".into()))?);
            let mut iter = buffer.chunks_exact(L);
            for bytes in iter.by_ref() {
                let item: R = rkyv::from_bytes(bytes)
                    .map_err(|_| Error::Stream("deserialization error".into()))?;
                filter_collect(&item);
            }
            remainder.extend_from_slice(iter.remainder());
        }
        Ok(())
    }

    /// Collects all items and returns them in a vector,
    /// returns empty vector if no items were present.
    pub fn collect_all<R, const L: usize>(
        mut stream: impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>> + std::marker::Unpin,
    ) -> Result<Vec<R>, Error>
    where
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>
            + Deserialize<R, SharedDeserializeMap>,
    {
        let mut vec = vec![];
        let mut buffer = vec![];
        while let Some(http_chunk) = stream.next().wait() {
            buffer.extend_from_slice(
                &http_chunk.map_err(|_| Error::Stream("chunking error".into()))?,
            );
            let mut chunk = buffer.chunks_exact(L);
            for bytes in chunk.by_ref() {
                let item: R = rkyv::from_bytes(bytes)
                    .map_err(|_| Error::Stream("deserialization error".into()))?;
                vec.push(item);
            }
            buffer = chunk.remainder().to_vec();
        }
        Ok(vec)
    }
}
