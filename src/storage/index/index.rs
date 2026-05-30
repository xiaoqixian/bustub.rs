//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// index.rs
//
// Identification: src/storage/index/index.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::sync::Arc;

use crate::catalog::Schema;
use crate::common::rid::RID;
use crate::concurrency::Transaction;
use crate::storage::table::tuple::Tuple;

/**
 * IndexMetadata - Holds metadata of an index object.
 *
 * The metadata object maintains the tuple schema and key attribute of an
 * index, since the external callers does not know the actual structure of
 * the index key, so it is the index's responsibility to maintain such a
 * mapping relation and does the conversion between tuple key and index key.
 */
pub struct IndexMetadata {
    /// The name of the index.
    name: String,
    /// The name of the table on which the index is created.
    table_name: String,
    /// The mapping relation between key schema and tuple schema.
    key_attrs: Vec<usize>,
    /// The schema of the indexed key.
    key_schema: Arc<Schema>,
    /// Is primary key?
    is_primary_key: bool,
}

impl IndexMetadata {
    /// Construct a new IndexMetadata instance.
    pub fn new(
        index_name: String,
        table_name: String,
        tuple_schema: &Schema,
        key_attrs: Vec<usize>,
        is_primary_key: bool,
    ) -> Self {
        let key_schema = Arc::new(Schema::copy_schema(tuple_schema, &key_attrs));
        IndexMetadata {
            name: index_name,
            table_name,
            key_attrs,
            key_schema,
            is_primary_key,
        }
    }

    /// Get the name of the index.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the name of the table on which the index is created.
    pub fn get_table_name(&self) -> &str {
        &self.table_name
    }

    /// Get a reference to the key schema.
    pub fn get_key_schema(&self) -> &Schema {
        &self.key_schema
    }

    pub fn clone_key_schema(&self) -> Arc<Schema> {
        self.key_schema.clone()
    }

    /// Get the number of columns inside index key.
    pub fn get_index_column_count(&self) -> usize {
        self.key_attrs.len()
    }

    /// Get the mapping relation between indexed columns and base table columns.
    pub fn get_key_attrs(&self) -> &[usize] {
        &self.key_attrs
    }

    /// Check if this is a primary key index.
    pub fn is_primary_key(&self) -> bool {
        self.is_primary_key
    }

    /// Return a string representation for debugging.
    pub fn to_string(&self) -> String {
        format!(
            "IndexMetadata[Name = {}, Type = B+Tree, Table name = {}] :: {}",
            self.name,
            self.table_name,
            self.key_schema.to_string_simplified(true),
        )
    }
}

/////////////////////////////////////////////////////////////////////
// Index trait definition
/////////////////////////////////////////////////////////////////////

/**
 * Index - Base trait for derived indices of different types.
 *
 * The index structure majorly maintains information on the schema of the
 * underlying table and the mapping relation between index key
 * and tuple key, and provides an abstracted way for the external world to
 * interact with the underlying index implementation without exposing
 * the actual implementation's interface.
 */
pub trait Index {
    /// Return a reference to the metadata object associated with the index.
    fn get_metadata(&self) -> &IndexMetadata;

    /// Return the number of indexed columns.
    fn get_index_column_count(&self) -> usize {
        self.get_metadata().get_index_column_count()
    }

    /// Return the index name.
    fn get_name(&self) -> &str {
        self.get_metadata().get_name()
    }

    /// Return the index key schema.
    fn get_key_schema(&self) -> &Schema {
        self.get_metadata().get_key_schema()
    }

    /// Return the index key attributes.
    fn get_key_attrs(&self) -> &[usize] {
        self.get_metadata().get_key_attrs()
    }

    /// Return a string representation for debugging.
    fn to_string(&self) -> String {
        format!("INDEX: ({}){}", self.get_name(), self.get_metadata().to_string())
    }

    ///////////////////////////////////////////////////////////////////
    // Point Modification
    ///////////////////////////////////////////////////////////////////

    /// Insert an entry into the index.
    fn insert_entry(&self, key: &Tuple, rid: RID, transaction: Option<&Transaction>) -> bool;

    /// Delete an index entry by key.
    fn delete_entry(&self, key: &Tuple, rid: RID, transaction: Option<&Transaction>);

    /// Search the index for the provided key.
    fn scan_key(&self, key: &Tuple, result: &mut Vec<RID>, transaction: Option<&Transaction>);
}
