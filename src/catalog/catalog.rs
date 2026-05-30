//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// catalog.rs
//
// Identification: src/catalog/catalog.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::Arc;

use crate::catalog::Schema;
use crate::storage::index::index::Index;

pub type IndexOid = u32;

pub enum IndexType {
    BPlusTreeIndex
}

pub struct IndexInfo {
    schema: Arc<Schema>,
    name: String,
    index: Box<dyn Index>,
    index_oid: IndexOid,
    key_size: usize,
    is_primary_key: bool,
    index_type: IndexType
}
