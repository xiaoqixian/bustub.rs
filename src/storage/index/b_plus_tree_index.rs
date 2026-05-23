// Date:   Sat May 23 17:28:42 2026
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian
//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// b_plus_tree_index.rs
//
// Identification: src/storage/index/b_plus_tree_index.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::cmp::Ordering;
use std::sync::Mutex;

use crate::buffer::buffer_pool_manager::BufferPoolManager;
use crate::catalog::schema::Schema;
use crate::common::rid::RID;
use crate::concurrency::Transaction;
use crate::sql_type::type_id::TypeId;
use crate::sql_type::value::Value;
use crate::storage::index::b_plus_tree::BPlusTree;
use crate::storage::index::index::{Index, IndexMetadata};
use crate::storage::table::tuple::Tuple;

fn vec_u8_comparator(a: &Vec<u8>, b: &Vec<u8>) -> Ordering {
    a.cmp(b)
}

/**
 * BPlusTreeIndex - A B+Tree based index that implements the `Index` trait.
 *
 * The index stores key-value pairs where the key is a byte-encoded
 * representation of the indexed columns (encoded to preserve sort order)
 * and the value is the RID of the corresponding tuple.
 */
pub struct BPlusTreeIndex {
    /// Index metadata (name, schema, key attrs, etc.).
    metadata: Box<IndexMetadata>,
    /// The underlying B+Tree container, protected by a mutex for interior mutability.
    container: Mutex<BPlusTree<Vec<u8>, RID, fn(&Vec<u8>, &Vec<u8>) -> Ordering>>,
}

impl BPlusTreeIndex {
    /// Default leaf node max size.
    const DEFAULT_LEAF_MAX_SIZE: usize = 100;
    /// Default internal node max size.
    const DEFAULT_INTERNAL_MAX_SIZE: usize = 100;

    /// Create a new B+Tree index.
    pub fn new(
        metadata: Box<IndexMetadata>,
        bpm: BufferPoolManager,
    ) -> Self {
        let header_page_id = bpm.new_page();
        let container = BPlusTree::new(
            metadata.get_name().to_string(),
            bpm,
            header_page_id,
            Self::DEFAULT_LEAF_MAX_SIZE,
            Self::DEFAULT_INTERNAL_MAX_SIZE,
            vec_u8_comparator as fn(&Vec<u8>, &Vec<u8>) -> Ordering,
        );
        BPlusTreeIndex {
            metadata,
            container: Mutex::new(container),
        }
    }

    /// Create a new B+Tree index with custom node sizes.
    pub fn new_with_sizes(
        metadata: Box<IndexMetadata>,
        bpm: BufferPoolManager,
        leaf_max_size: usize,
        internal_max_size: usize,
    ) -> Self {
        let header_page_id = bpm.new_page();
        let container = BPlusTree::new(
            metadata.get_name().to_string(),
            bpm,
            header_page_id,
            leaf_max_size,
            internal_max_size,
            vec_u8_comparator as fn(&Vec<u8>, &Vec<u8>) -> Ordering,
        );
        BPlusTreeIndex {
            metadata,
            container: Mutex::new(container),
        }
    }

    /// Extract a sort-key-encoded byte array from a key tuple using the given schema.
    ///
    /// Each column value is encoded into a byte sequence that, when concatenated,
    /// preserves the correct SQL sort order under lexicographic byte comparison.
    fn extract_key(key: &Tuple, key_schema: &Schema) -> Vec<u8> {
        let mut encoded = Vec::new();
        for i in 0..key_schema.get_column_count() {
            let value = key.get_value(key_schema, i);
            encoded.extend_from_slice(&Self::encode_value(&value));
        }
        encoded
    }

    /// Encode a single Value into bytes that preserve sort order
    /// under lexicographic byte comparison.
    fn encode_value(value: &Value) -> Vec<u8> {
        match value.get_type_id() {
            TypeId::Boolean | TypeId::TinyInt => {
                // u8 sorts correctly as-is
                vec![value.get_as::<u8>()]
            }
            TypeId::SmallInt => {
                // Big-endian with sign bit flipped so negative < positive
                let v = value.get_as::<i16>();
                let encoded = (v as u16) ^ 0x8000;
                encoded.to_be_bytes().to_vec()
            }
            TypeId::Integer => {
                let v = value.get_as::<i32>();
                let encoded = (v as u32) ^ 0x8000_0000;
                encoded.to_be_bytes().to_vec()
            }
            TypeId::BigInt => {
                let v = value.get_as::<i64>();
                let encoded = (v as u64) ^ 0x8000_0000_0000_0000;
                encoded.to_be_bytes().to_vec()
            }
            TypeId::Timestamp => {
                // u64 sorts correctly in big-endian
                value.get_as::<u64>().to_be_bytes().to_vec()
            }
            TypeId::Decimal => {
                // IEEE 754 total-order encoding:
                //   positive: flip sign bit (makes positive > negative lexicographically)
                //   negative: flip all bits (larger magnitude → smaller lexicographically)
                let v = value.get_as::<f64>();
                let bits = v.to_bits();
                let encoded = if v.is_sign_positive() {
                    bits ^ 0x8000_0000_0000_0000
                } else {
                    bits ^ 0xFFFF_FFFF_FFFF_FFFF
                };
                encoded.to_be_bytes().to_vec()
            }
            TypeId::Varchar | TypeId::Vector => {
                // String/varchar data sorts correctly as raw bytes (UTF-8 / binary).
                value.get_data().to_vec()
            }
            TypeId::Invalid => panic!("Cannot encode INVALID value"),
        }
    }
}

impl Index for BPlusTreeIndex {
    fn get_metadata(&self) -> &IndexMetadata {
        &self.metadata
    }

    fn insert_entry(&self, key: &Tuple, rid: RID, _transaction: Option<&Transaction>) -> bool {
        let key_schema = self.metadata.get_key_schema();
        let index_key = Self::extract_key(key, key_schema);
        self.container.lock().unwrap().insert(index_key, rid);
        true
    }

    fn delete_entry(&self, key: &Tuple, _rid: RID, _transaction: Option<&Transaction>) {
        let key_schema = self.metadata.get_key_schema();
        let index_key = Self::extract_key(key, key_schema);
        self.container.lock().unwrap().remove(&index_key);
    }

    fn scan_key(&self, key: &Tuple, result: &mut Vec<RID>, _transaction: Option<&Transaction>) {
        let key_schema = self.metadata.get_key_schema();
        let index_key = Self::extract_key(key, key_schema);
        if let Some(&rid) = self.container.lock().unwrap().get_value(&index_key) {
            result.push(rid);
        }
    }
}
