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

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crate::buffer::buffer_pool_manager::BufferPoolManager;
use crate::catalog::schema::Schema;
use crate::common::rid::RID;
use crate::concurrency::Transaction;
use crate::storage::index::b_plus_tree_index::new_gk_b_plus_tree_index;
use crate::storage::index::index::{Index, IndexMetadata};
use crate::storage::table::table_heap::TableHeap;

/// Type alias for table OID.
pub type TableOid = u32;

/// Type alias for column OID.
pub type ColumnOid = u32;

/// Type alias for index OID.
pub type IndexOid = u32;

/// Index type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    BPlusTreeIndex,
}

/// TableInfo maintains metadata about a table.
pub struct TableInfo {
    /// The table schema.
    pub schema: Schema,
    /// The table name.
    pub name: String,
    /// The table heap.
    pub table: TableHeap,
    /// The table OID.
    pub oid: TableOid,
}

impl TableInfo {
    /// Construct a new TableInfo instance.
    pub fn new(schema: Schema, name: String, table: TableHeap, oid: TableOid) -> Self {
        TableInfo {
            schema,
            name,
            table,
            oid,
        }
    }
}

/// IndexInfo maintains metadata about an index.
pub struct IndexInfo {
    /// The schema for the index key.
    pub key_schema: Schema,
    /// The name of the index.
    pub name: String,
    /// An owning pointer to the index.
    pub index: Box<dyn Index>,
    /// The unique OID for the index.
    pub index_oid: IndexOid,
    /// The name of the table on which the index is created.
    pub table_name: String,
    /// The size of the index key, in bytes.
    pub key_size: usize,
    /// Is primary key index?
    pub is_primary_key: bool,
    /// The index type.
    pub index_type: IndexType,
}

impl IndexInfo {
    /// Construct a new IndexInfo instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key_schema: Schema,
        name: String,
        index: Box<dyn Index>,
        index_oid: IndexOid,
        table_name: String,
        key_size: usize,
        is_primary_key: bool,
        index_type: IndexType,
    ) -> Self {
        IndexInfo {
            key_schema,
            name,
            index,
            index_oid,
            table_name,
            key_size,
            is_primary_key,
            index_type,
        }
    }
}

/// The Catalog handles table creation, table lookup, index creation, and index lookup.
pub struct Catalog {
    /// Map table identifier -> table metadata.
    tables: HashMap<TableOid, Arc<TableInfo>>,
    /// Map table name -> table identifiers.
    table_names: HashMap<String, TableOid>,
    /// The next table identifier to be used.
    next_table_oid: AtomicU32,
    /// Map index identifier -> index metadata.
    indexes: HashMap<IndexOid, Arc<IndexInfo>>,
    /// Map table name -> index names -> index identifiers.
    index_names: HashMap<String, HashMap<String, IndexOid>>,
    /// The next index identifier to be used.
    next_index_oid: AtomicU32,

    bpm: Arc<BufferPoolManager>,
}

impl Catalog {
    /// Construct a new Catalog instance.
    pub fn new(bpm: Arc<BufferPoolManager>) -> Self {
        Catalog {
            tables: HashMap::new(),
            table_names: HashMap::new(),
            next_table_oid: AtomicU32::new(0),
            indexes: HashMap::new(),
            index_names: HashMap::new(),
            next_index_oid: AtomicU32::new(0),
            bpm,
        }
    }

    /// Create a new table and return its metadata.
    ///
    /// When `create_table_heap` is false, an empty table heap is created (used
    /// for binder tests or when running without a buffer pool).
    pub fn create_table(
        &mut self,
        _txn: Option<&Transaction>,
        table_name: &str,
        schema: &Schema,
        create_table_heap: bool,
    ) -> Option<Arc<TableInfo>> {
        // Reject duplicate table names.
        if self.table_names.contains_key(table_name) {
            return None;
        }

        // Construct the table heap.
        let table = if create_table_heap {
            TableHeap::new(self.bpm.clone())
        } else {
            TableHeap::create_empty_heap()
        };

        // Fetch the table OID for the new table.
        let table_oid = self.next_table_oid.fetch_add(1, Ordering::Relaxed);

        // Construct the table information.
        let schema_copy = Schema::new(schema.get_columns().clone());
        let meta = Arc::new(TableInfo::new(schema_copy, table_name.to_string(), table, table_oid));

        // Update internal tracking.
        self.tables.insert(table_oid, meta.clone());
        self.table_names.insert(table_name.to_string(), table_oid);
        self.index_names.insert(table_name.to_string(), HashMap::new());

        Some(meta)
    }

    /// Query table metadata by name.
    pub fn get_table_by_name(&self, table_name: &str) -> Option<Arc<TableInfo>> {
        let table_oid = self.table_names.get(table_name)?;
        self.tables.get(table_oid).cloned()
    }

    pub fn get_table_ref_by_name(&self, table_name: &str) -> Option<&TableInfo> {
        let table_oid = self.table_names.get(table_name)?;
        self.tables.get(table_oid).map(|v| v.as_ref())
    }

    /// Query table metadata by OID.
    pub fn get_table_by_oid(&self, table_oid: TableOid) -> Option<Arc<TableInfo>> {
        self.tables.get(&table_oid).cloned()
    }

    /// Create a new B+Tree index on the specified table, populate existing data,
    /// and return its metadata.
    ///
    /// Only B+Tree index is currently supported.
    pub fn create_index(
        &mut self,
        _txn: Option<&Transaction>,
        index_name: &str,
        table_name: &str,
        schema: &Schema,
        key_schema: &Schema,
        key_attrs: &[usize],
        keysize: usize,
        is_primary_key: bool,
        index_type: IndexType,
    ) -> Option<Arc<IndexInfo>> {
        // Reject creation request for nonexistent table.
        if !self.table_names.contains_key(table_name) {
            return None;
        }

        // Determine if the requested index already exists for this table.
        let table_indexes = self.index_names.get_mut(table_name).unwrap();
        if table_indexes.contains_key(index_name) {
            return None;
        }

        // Construct index metadata.
        // key_attrs are the indices of the key columns in the table schema.
        let index_meta = IndexMetadata::new(
            index_name.to_string(),
            table_name.to_string(),
            schema,
            key_attrs.to_vec(),
            is_primary_key,
        );

        // Construct the B+Tree index (the only supported index type).
        let index = match index_type {
            IndexType::BPlusTreeIndex => Box::new(new_gk_b_plus_tree_index::<8, RID>(
                index_meta,
                self.bpm.clone(),
            ))
        };

        // Populate the index with all tuples in the table heap.
        let table_meta = {
            match self.table_names.get(table_name) {
                None => None,
                Some(table_oid) => {
                    self.tables.get(table_oid).map(|v| v.as_ref())
                }
            }
        }.expect(format!("table {} not found", table_name).as_str());

        let mut iter = table_meta.table.make_iterator();
        while !iter.is_end() {
            let (_meta, tuple) = iter.get_tuple();
            let index_key = tuple.key_from_tuple(schema, key_schema, key_attrs);
            index.insert_entry(&index_key, tuple.get_rid(), _txn);
            iter.next();
        }

        // Get the next OID for the new index.
        let index_oid = self.next_index_oid.fetch_add(1, Ordering::Relaxed);

        // Construct index information; IndexInfo takes ownership of the index.
        let key_schema_copy = Schema::new(key_schema.get_columns().clone());
        let index_info = Arc::new(IndexInfo::new(
            key_schema_copy,
            index_name.to_string(),
            index,
            index_oid,
            table_name.to_string(),
            keysize,
            is_primary_key,
            IndexType::BPlusTreeIndex,
        ));

        // Update internal tracking.
        self.indexes.insert(index_oid, index_info.clone());
        table_indexes.insert(index_name.to_string(), index_oid);

        Some(index_info)
    }

    /// Get the index by name and table name.
    pub fn get_index_by_name(&self, index_name: &str, table_name: &str) -> Option<Arc<IndexInfo>> {
        let table = self.index_names.get(table_name)?;
        let index_oid = table.get(index_name)?;
        self.indexes.get(index_oid).cloned()
    }

    /// Get the index by name and table OID.
    pub fn get_index_by_name_and_table_oid(
        &self,
        index_name: &str,
        table_oid: TableOid,
    ) -> Option<Arc<IndexInfo>> {
        let table_meta = self.tables.get(&table_oid)?;
        self.get_index_by_name(index_name, &table_meta.name)
    }

    /// Get the index by index OID.
    pub fn get_index_by_oid(&self, index_oid: IndexOid) -> Option<Arc<IndexInfo>> {
        self.indexes.get(&index_oid).cloned()
    }

    /// Get all indexes for the table identified by `table_name`.
    pub fn get_table_indexes(&self, table_name: &str) -> Vec<Arc<IndexInfo>> {
        let table_indexes = match self.index_names.get(table_name) {
            Some(v) => v,
            None => return Vec::new(),
        };

        table_indexes
            .values()
            .filter_map(|oid| self.indexes.get(oid).cloned())
            .collect()
    }

    /// Get all table names.
    pub fn get_table_names(&self) -> Vec<String> {
        self.table_names.keys().cloned().collect()
    }
}
