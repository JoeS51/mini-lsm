// Copyright (c) 2022-2025 Alex Chi Z
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::Bytes;
pub use iterator::BlockIterator;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the course
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.data);
        for offset in &self.offsets {
            buf.extend_from_slice(&offset.to_le_bytes())
        }
        let num_elements = self.offsets.len() as u16;
        buf.extend_from_slice(&num_elements.to_le_bytes());
        Bytes::from(buf)
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        let len = data.len();
        let n_bytes = [data[len - 2], data[len - 1]];
        let n = u16::from_le_bytes(n_bytes);
        let offsets = &data[len - 2 - n as usize * 2..len - 2];

        let mut block_offsets: Vec<u16> = Vec::new();

        for i in (1..offsets.len()).step_by(2) {
            let bytes = [offsets[i - 1], offsets[i]];
            let offset = u16::from_le_bytes(bytes);
            block_offsets.push(offset);
        }

        let data_section_end = len - 2 - n as usize * 2;
        let block_data: Vec<u8> = data[..data_section_end].to_vec();

        Self {
            data: block_data,
            offsets: block_offsets,
        }
    }
}
