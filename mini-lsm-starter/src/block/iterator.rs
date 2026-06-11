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

use std::sync::Arc;

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// Iterates on a block.
pub struct BlockIterator {
    /// The internal `Block`, wrapped by an `Arc`
    block: Arc<Block>,
    /// The current key, empty represents the iterator is invalid
    key: KeyVec,
    /// the current value range in the block.data, corresponds to the current key
    value_range: (usize, usize),
    /// Current index of the key-value pair, should be in range of [0, num_of_elements)
    idx: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
            first_key: KeyVec::new(),
        }
    }

    /// Creates a block iterator and seek to the first entry.
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        let mut block_iter = BlockIterator::new(block.clone());
        block_iter.seek_to_first();
        block_iter
    }

    /// Creates a block iterator and seek to the first key that >= `key`.
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        unimplemented!()
    }

    /// Returns the key of the current entry.
    pub fn key(&self) -> KeySlice<'_> {
        self.key.as_key_slice()
    }

    /// Returns the value of the current entry.
    pub fn value(&self) -> &[u8] {
        &self.block.data[self.value_range.0..self.value_range.1]
    }

    /// Returns true if the iterator is valid.
    /// Note: You may want to make use of `key`
    pub fn is_valid(&self) -> bool {
        self.idx < self.block.offsets.len()
    }

    /// Seeks to the first key in the block.
    pub fn seek_to_first(&mut self) {
        self.idx = 0;
        let (key_vec, start, end) = {
            let (key, _, (start, end)) = self.get_block_entry(self.idx);
            (key.to_vec(), start, end)
        };
        self.key = KeyVec::from_vec(key_vec.to_vec());
        self.first_key = KeyVec::from_vec(key_vec.to_vec());
        self.value_range = (start, end);
    }

    fn get_block_entry(&self, index: usize) -> (&[u8], &[u8], (usize, usize)) {
        let offset = self.block.offsets[index] as usize;
        let key_size_bytes = [self.block.data[offset], self.block.data[offset + 1]];
        let key_size = u16::from_le_bytes(key_size_bytes) as usize;
        let key_bytes = &self.block.data[offset + 2..offset + 2 + key_size];

        let value_offset = offset + 2 + key_size;
        let value_size_bytes = [
            self.block.data[value_offset],
            self.block.data[value_offset + 1],
        ];
        let value_size = u16::from_le_bytes(value_size_bytes) as usize;
        let value_bytes = &self.block.data[value_offset + 2..value_offset + 2 + value_size];

        (
            key_bytes,
            value_bytes,
            (value_offset + 2, value_offset + 2 + value_size),
        )
    }

    /// Move to the next key in the block.
    pub fn next(&mut self) {
        self.idx += 1;
        if !self.is_valid() {
            self.key = KeyVec::from_vec(Vec::new());
            return;
        }
        let (key, val, (start, end)) = self.get_block_entry(self.idx);
        self.key = KeyVec::from_vec(key.to_vec());
        self.value_range = (start, end);
    }

    /// Seek to the first key that >= `key`.
    /// Note: You should assume the key-value pairs in the block are sorted when being added by
    /// callers.
    pub fn seek_to_key(&mut self, key: KeySlice) {
        let mut lo = 0;
        let mut hi = self.block.offsets.len();
        while lo < hi {
            let mid_idx = (lo + hi) / 2;
            let (mid_key, _, (_, _)) = self.get_block_entry(mid_idx);
            if KeySlice::from_slice(mid_key) < key {
                lo = mid_idx + 1;
            } else {
                hi = mid_idx;
            }
        }
        if lo == self.block.offsets.len() {
            self.key = KeyVec::from_vec(Vec::new());
            return;
        }
        self.idx = lo;
        let (key_vec, start, end) = {
            let (key, _, (start, end)) = self.get_block_entry(self.idx);
            (key.to_vec(), start, end)
        };
        self.key = KeyVec::from_vec(key_vec.to_vec());
        self.first_key = KeyVec::from_vec(key_vec.to_vec());
        self.value_range = (start, end);
    }
}
