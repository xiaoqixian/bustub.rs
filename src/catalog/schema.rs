//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// schema.rs
//
// Identification: src/catalog/schema.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::rc::Rc;

use super::column::Column;

pub type SchemaRef = Rc<Schema>;

/// A Schema represents the schema (column types, names, offsets, etc.)
/// for a table. It is composed of Column objects.
#[derive(Clone)]
pub struct Schema {
    /// Fixed-length column size, i.e. the number of bytes used by one tuple.
    length: usize,
    /// All the columns in the schema, inlined and uninlined.
    pub(crate) columns: Vec<Column>,
    /// True if all the columns are inlined, false otherwise.
    tuple_is_inlined: bool,
    /// Indices of all uninlined columns.
    uninlined_columns: Vec<usize>,
}

impl Schema {
    /// Constructs the schema corresponding to the vector of columns,
    /// read left-to-right.
    pub fn new(columns: Vec<Column>) -> Self {
        let mut curr_offset = 0usize;
        let mut tuple_is_inlined = true;
        let mut uninlined_columns = Vec::new();
        let mut schema_columns = Vec::with_capacity(columns.len());

        for (index, mut column) in columns.into_iter().enumerate() {
            // handle uninlined column
            if !column.is_inlined() {
                tuple_is_inlined = false;
                uninlined_columns.push(index);
            }
            // set column offset
            column.column_offset = curr_offset;
            if column.is_inlined() {
                curr_offset += column.get_storage_size();
            } else {
                curr_offset += size_of::<usize>();
            }

            // add column
            schema_columns.push(column);
        }

        Schema {
            length: curr_offset,
            columns: schema_columns,
            tuple_is_inlined,
            uninlined_columns,
        }
    }

    /// Create a new Schema by copying a subset of columns from an existing schema.
    pub fn copy_schema(from: &Schema, attrs: &[usize]) -> Self {
        let cols: Vec<Column> = attrs
            .iter()
            .map(|&i| from.columns[i].clone())
            .collect();
        Schema::new(cols)
    }

    /// Get all the columns in the schema.
    pub fn get_columns(&self) -> &Vec<Column> {
        &self.columns
    }

    /// Get a specific column from the schema by index.
    pub fn get_column(&self, col_idx: usize) -> &Column {
        &self.columns[col_idx]
    }

    /// Look up the index of a column by name. Panics if not found.
    pub fn get_col_idx(&self, col_name: &str) -> u32 {
        self.try_get_col_idx(col_name)
            .expect("Column does not exist")
    }

    /// Look up the index of a column by name. Returns None if not found.
    pub fn try_get_col_idx(&self, col_name: &str) -> Option<u32> {
        for (i, col) in self.columns.iter().enumerate() {
            if col.get_name() == col_name {
                return Some(i as u32);
            }
        }
        None
    }

    /// Get the indices of non-inlined columns.
    pub fn get_unlined_columns(&self) -> &Vec<usize> {
        &self.uninlined_columns
    }

    /// Get the number of columns in the schema.
    pub fn get_column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get the number of non-inlined columns.
    pub fn get_unlined_column_count(&self) -> u32 {
        self.uninlined_columns.len() as u32
    }

    /// Get the number of bytes used by one tuple.
    pub fn get_inlined_storage_size(&self) -> usize {
        self.length
    }

    /// Check if all columns are inlined.
    pub fn is_inlined(&self) -> bool {
        self.tuple_is_inlined
    }
}

impl std::fmt::Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_simplified(true))
    }
}

impl Schema {
    /// Get a string representation of this schema.
    pub fn to_string_simplified(&self, simplified: bool) -> String {
        if simplified {
            let mut s = String::from("(");
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&col.to_string_simplified(true));
            }
            s.push(')');
            return s;
        }

        let mut s = format!(
            "Schema[NumColumns:{}, IsInlined:{}, Length:{}] :: (",
            self.get_column_count(),
            self.tuple_is_inlined,
            self.length,
        );
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&col.to_string_simplified(false));
        }
        s.push(')');
        s
    }
}


