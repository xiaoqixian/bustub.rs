//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// column.rs
//
// Identification: src/catalog/column.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use crate::sql_type::type_id::TypeId;
use crate::sql_type::sql_type::type_id_to_string;

/// A Column represents a single column in a table schema.
#[derive(Clone)]
pub struct Column {
    /// Column name.
    pub(crate) column_name: String,
    /// Column value's type.
    pub(crate) column_type: TypeId,
    /// The size of the column (in bytes). For fixed-length types this is the
    /// type size; for variable-length types this is the specified max length.
    pub(crate) length: usize,
    /// Column offset in the tuple.
    pub(crate) column_offset: usize,
}

impl Column {
    /// Non-variable-length constructor for creating a Column.
    pub fn new(column_name: String, column_type: TypeId) -> Self {
        assert!(
            column_type != TypeId::Varchar,
            "Wrong constructor for VARCHAR type."
        );
        let length = Self::type_size(column_type, 0) as usize;
        Column {
            column_name,
            column_type,
            length,
            column_offset: 0,
        }
    }

    /// Variable-length constructor for creating a Column.
    pub fn new_with_length(column_name: String, column_type: TypeId, length: u32) -> Self {
        assert!(
            column_type == TypeId::Varchar,
            "Wrong constructor for fixed-size type."
        );
        let length = Self::type_size(column_type, length) as usize;
        Column {
            column_name,
            column_type,
            length,
            column_offset: 0,
        }
    }

    /// Replicate a Column with a different name.
    pub fn new_with_name(column_name: String, column: &Column) -> Self {
        Column {
            column_name,
            column_type: column.column_type,
            length: column.length,
            column_offset: column.column_offset,
        }
    }

    /// Return a copy of this column with a different name.
    pub fn with_column_name(&self, column_name: String) -> Self {
        Column {
            column_name,
            column_type: self.column_type,
            length: self.length,
            column_offset: self.column_offset,
        }
    }

    /// Get the column name.
    pub fn get_name(&self) -> &str {
        &self.column_name
    }

    /// Get the storage size of this column.
    pub fn get_storage_size(&self) -> usize {
        self.length
    }

    /// Get the column's offset in the tuple.
    pub fn get_offset(&self) -> usize {
        self.column_offset
    }

    /// Get the column type.
    pub fn get_type(&self) -> TypeId {
        self.column_type
    }

    /// Check if the column is inlined (i.e., not VARCHAR or VECTOR).
    pub fn is_inlined(&self) -> bool {
        self.column_type != TypeId::Varchar
    }

    /// Return the size in bytes of the type.
    fn type_size(type_id: TypeId, length: u32) -> u8 {
        match type_id {
            TypeId::Boolean | TypeId::TinyInt => 1,
            TypeId::SmallInt => 2,
            TypeId::Integer => 4,
            TypeId::BigInt | TypeId::Decimal | TypeId::Timestamp => 8,
            TypeId::Varchar => length as u8,
        }
    }
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_simplified(true))
    }
}

impl Column {
    /// Get a string representation of this column.
    pub fn to_string_simplified(&self, simplified: bool) -> String {
        if simplified {
            let mut s = format!("{}:{}", self.column_name, type_id_to_string(self.column_type));
            if self.column_type == TypeId::Varchar {
                s.push_str(&format!("({})", self.length));
            }
            return s;
        }

        format!(
            "Column[{}, {}, Offset:{}, Length:{}]",
            self.column_name,
            type_id_to_string(self.column_type),
            self.column_offset,
            self.length,
        )
    }
}
