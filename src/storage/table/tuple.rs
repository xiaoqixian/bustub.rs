//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// tuple.rs
//
// Identification: src/storage/table/tuple.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::catalog::Schema;
use crate::common::rid::RID;
use crate::sql_type::limits::BUSTUB_VALUE_NULL;
use crate::sql_type::value::Value;

/// Timestamp type used in tuple metadata.
pub type TimeStamp = i64;

/// Invalid timestamp constant.
pub const INVALID_TS: TimeStamp = -1;

/// The size of TupleMeta in bytes (16 bytes: 8 for ts + 1 for bool + padding).
pub const TUPLE_META_SIZE: usize = 16;

/// Metadata associated with a tuple stored in a table heap.
/// In project 3, simply set ts to 0 and is_deleted to false.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TupleMeta {
    /// The timestamp / transaction ID of this tuple.
    pub ts: TimeStamp,
    /// Marks whether this tuple is marked removed from the table heap.
    pub is_deleted: bool,
}

/// Tuple format:
/// ---------------------------------------------------------------------
/// | FIXED-SIZE or VARIED-SIZED OFFSET | PAYLOAD OF VARIED-SIZED FIELD |
/// ---------------------------------------------------------------------
#[derive(Clone)]
pub struct Tuple {
    /// The RID associated with this tuple (valid when pointing to the table heap).
    pub(crate) rid: RID,
    /// The raw byte data of the tuple.
    pub(crate) data: Vec<u8>,
}

impl Tuple {
    /// Create a default (dummy) tuple.
    pub fn new() -> Self {
        Tuple {
            rid: RID::new(),
            data: Vec::new(),
        }
    }

    /// Create a tuple with a specific RID (for table heap).
    pub fn new_with_rid(rid: RID) -> Self {
        Tuple {
            rid,
            data: Vec::new(),
        }
    }

    /// Create an empty tuple with an invalid RID.
    pub fn empty() -> Self {
        Tuple::new_with_rid(RID::new())
    }

    /// Create a new tuple from a list of Values and a Schema.
    pub fn new_with_values(values: Vec<Value>, schema: &Schema) -> Self {
        assert_eq!(values.len(), schema.get_column_count() as usize);

        // 1. Calculate the size of the tuple.
        let mut tuple_size = schema.get_inlined_storage_size();
        for &i in schema.get_unlined_columns() {
            let mut len = values[i].get_storage_size();
            if len == BUSTUB_VALUE_NULL {
                len = 0;
            }
            tuple_size += size_of::<u32>() + len as usize;
        }

        // 2. Allocate memory.
        let mut data = vec![0u8; tuple_size as usize];

        // 3. Serialize each attribute based on the input value.
        let column_count = schema.get_column_count();
        let mut offset = schema.get_inlined_storage_size();

        for i in 0..column_count {
            let col = schema.get_column(i);
            if !col.is_inlined() {
                // Serialize relative offset, where the actual varchar data is stored.
                let off = col.get_offset() as usize;
                data[off..off + 4].copy_from_slice(&offset.to_le_bytes());
                // Serialize varchar value, in place (size + data).
                values[i as usize].serialize_to(&mut data[offset as usize..]);
                let mut len = values[i as usize].get_storage_size();
                if len == BUSTUB_VALUE_NULL {
                    len = 0;
                }
                offset += size_of::<u32>() + len as usize;
            } else {
                let off = col.get_offset() as usize;
                values[i as usize].serialize_to(&mut data[off..]);
            }
        }

        Tuple {
            rid: RID::new(),
            data,
        }
    }

    /// Create a tuple from raw bytes.
    pub fn new_with_data(rid: RID, data: &[u8], size: u32) -> Self {
        Tuple {
            rid,
            data: data[..size as usize].to_vec(),
        }
    }

    /// Serialize the tuple data (size + content) into a byte buffer.
    /// Format: [4-byte size][size bytes of data].
    pub fn serialize_to(&self, storage: &mut [u8]) {
        let sz = self.data.len() as i32;
        storage[..4].copy_from_slice(&sz.to_le_bytes());
        storage[4..4 + sz as usize].copy_from_slice(&self.data);
    }

    /// Deserialize the tuple data from a byte buffer.
    /// Format: [4-byte size][size bytes of data].
    pub fn deserialize_from(&mut self, storage: &[u8]) {
        let size = u32::from_le_bytes(storage[..4].try_into().unwrap()) as usize;
        self.data.resize(size, 0);
        self.data.copy_from_slice(&storage[4..4 + size]);
    }

    /// Get the RID of this tuple.
    pub fn get_rid(&self) -> RID {
        self.rid
    }

    /// Set the RID of this tuple.
    pub fn set_rid(&mut self, rid: RID) {
        self.rid = rid;
    }

    /// Get a reference to the raw tuple data.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the tuple data in bytes.
    pub fn get_length(&self) -> u32 {
        self.data.len() as u32
    }

    /// Get the value of a specified column.
    pub fn get_value(&self, schema: &Schema, column_idx: usize) -> Value {
        let column_type = schema.get_column(column_idx).get_type();
        let data_ptr = self.get_data_ptr(schema, column_idx);
        Value::deserialize_from(data_ptr, column_type)
    }

    /// Generate a key tuple given schemas and key attributes.
    pub fn key_from_tuple(
        &self,
        schema: &Schema,
        key_schema: &Schema,
        key_attrs: &[usize],
    ) -> Self {
        let values: Vec<Value> = key_attrs
            .iter()
            .map(|&idx| self.get_value(schema, idx))
            .collect();
        Tuple::new_with_values(values, key_schema)
    }

    /// Check if a column value is null.
    pub fn is_null(&self, schema: &Schema, column_idx: usize) -> bool {
        let value = self.get_value(schema, column_idx);
        value.is_null()
    }

    /// Get a string representation of this tuple.
    pub fn to_string(&self, schema: &Schema) -> String {
        let column_count = schema.get_column_count();
        let mut parts = Vec::with_capacity(column_count as usize);
        for column_itr in 0..column_count {
            if self.is_null(schema, column_itr) {
                parts.push("<NULL>".to_string());
            } else {
                let val = self.get_value(schema, column_itr);
                parts.push(val.to_string_val());
            }
        }
        format!("({})", parts.join(", "))
    }

    /// Get the starting storage address of a specific column's data.
    fn get_data_ptr(&self, schema: &Schema, column_idx: usize) -> &[u8] {
        let col = schema.get_column(column_idx);
        if col.is_inlined() {
            let offset = col.get_offset() as usize;
            // Return a slice starting at the inline data position.
            return &self.data[offset..];
        }
        // Read the relative offset from the tuple data.
        let offset = col.get_offset() as usize;
        let relative_offset =
            i32::from_le_bytes(self.data[offset..offset + 4].try_into().unwrap()) as usize;
        // Return a slice starting at the real data position for the VARCHAR type.
        &self.data[relative_offset..]
    }
}

impl Default for Tuple {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two tuples have equal content.
pub fn is_tuple_content_equal(a: &Tuple, b: &Tuple) -> bool {
    a.data == b.data
}

