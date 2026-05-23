//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// value.rs
//
// Identification: src/sql_type/value.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::sql_type::{get_type_instance, CmpBool};
use super::type_id::TypeId;

/// A Value is an abstract representation of SQL data stored in some
/// materialized state. All values have a type and comparison functions,
/// but subclasses implement other type-specific functionality.
#[derive(Clone)]
pub struct Value {
    /// The actual raw value stored as a union-like representation.
    /// - `Boolean`/`TinyInt`: stored as `i8`
    /// - `SmallInt`: stored as `i16`
    /// - `Integer`: stored as `i32`
    /// - `BigInt`/`Timestamp`: stored as `i64`/`u64`
    /// - `Decimal`: stored as `f64`
    /// - `Varchar`/`Vector`: stored as `Vec<u8>`
    pub(crate) raw_value: ValuePayload,
    /// For variable-length types: the length of the data.
    /// Otherwise unused. If equal to BUSTUB_VALUE_NULL, the value is NULL.
    pub(crate) size_len: u32,
    /// Whether the variable-length data is owned by this value.
    pub(crate) manage_data: bool,
    /// The data type of this value.
    pub(crate) sql_type_id: TypeId,
}

/// Internal representation of a value payload, similar to a C++ union.
#[derive(Clone)]
pub(crate) enum ValuePayload {
    Boolean(i8),
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Decimal(f64),
    Timestamp(u64),
    Varlen(Vec<u8>),
    /// For VARCHAR values where data is not owned by this Value.
    VarlenRef,
    /// For invalid/null typed values.
    Nil,
}

impl Value {
    /// Create a new Value with the given type ID. The value is initialized as NULL.
    pub fn new(type_id: TypeId) -> Self {
        Value {
            raw_value: ValuePayload::Nil,
            size_len: BUSTUB_VALUE_NULL,
            manage_data: false,
            sql_type_id: type_id,
        }
    }

    /// Create a boolean or tinyint value.
    pub fn from_i8(type_id: TypeId, val: i8) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Boolean => {
                v.raw_value = ValuePayload::Boolean(val);
                v.size_len = if val == BUSTUB_BOOLEAN_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            TypeId::TinyInt => {
                v.raw_value = ValuePayload::TinyInt(val);
                v.size_len = if val == BUSTUB_INT8_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            _ => {
                panic!("Invalid Type for i8 Value constructor");
            }
        }
        v
    }

    /// Create a smallint value.
    pub fn from_i16(type_id: TypeId, val: i16) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::SmallInt => {
                v.raw_value = ValuePayload::SmallInt(val);
                v.size_len = if val == BUSTUB_INT16_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            _ => {
                v.raw_value = ValuePayload::Integer(val as i32);
                // Try mapping from the alternative constructors
                match type_id {
                    TypeId::Boolean => {
                        v.raw_value = ValuePayload::Boolean(val as i8);
                        v.size_len = if val as i8 == BUSTUB_BOOLEAN_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::TinyInt => {
                        v.raw_value = ValuePayload::TinyInt(val as i8);
                        v.size_len = if val as i8 == BUSTUB_INT8_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::Integer => {
                        v.raw_value = ValuePayload::Integer(val as i32);
                        v.size_len = if val as i32 == BUSTUB_INT32_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::BigInt => {
                        v.raw_value = ValuePayload::BigInt(val as i64);
                        v.size_len = if val as i64 == BUSTUB_INT64_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::Timestamp => {
                        v.raw_value = ValuePayload::Timestamp(val as u64);
                        v.size_len = if val as u64 == BUSTUB_TIMESTAMP_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    _ => {
                        panic!("Invalid Type for i16 Value constructor");
                    }
                }
            }
        }
        v
    }

    /// Create an integer value.
    pub fn from_i32(type_id: TypeId, val: i32) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Integer => {
                v.raw_value = ValuePayload::Integer(val);
                v.size_len = if val == BUSTUB_INT32_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            _ => {
                match type_id {
                    TypeId::Boolean => {
                        v.raw_value = ValuePayload::Boolean(val as i8);
                        v.size_len = if val as i8 == BUSTUB_BOOLEAN_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::TinyInt => {
                        v.raw_value = ValuePayload::TinyInt(val as i8);
                        v.size_len = if val as i8 == BUSTUB_INT8_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::SmallInt => {
                        v.raw_value = ValuePayload::SmallInt(val as i16);
                        v.size_len = if val as i16 == BUSTUB_INT16_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::BigInt => {
                        v.raw_value = ValuePayload::BigInt(val as i64);
                        v.size_len = if val as i64 == BUSTUB_INT64_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::Timestamp => {
                        v.raw_value = ValuePayload::Timestamp(val as u64);
                        v.size_len = if val as u64 == BUSTUB_TIMESTAMP_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    _ => {
                        panic!("Invalid Type for i32 Value constructor");
                    }
                }
            }
        }
        v
    }

    /// Create a bigint or timestamp value.
    pub fn from_i64(type_id: TypeId, val: i64) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::BigInt => {
                v.raw_value = ValuePayload::BigInt(val);
                v.size_len = if val == BUSTUB_INT64_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            _ => {
                match type_id {
                    TypeId::Boolean => {
                        v.raw_value = ValuePayload::Boolean(val as i8);
                        v.size_len = if val as i8 == BUSTUB_BOOLEAN_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::TinyInt => {
                        v.raw_value = ValuePayload::TinyInt(val as i8);
                        v.size_len = if val as i8 == BUSTUB_INT8_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::SmallInt => {
                        v.raw_value = ValuePayload::SmallInt(val as i16);
                        v.size_len = if val as i16 == BUSTUB_INT16_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::Integer => {
                        v.raw_value = ValuePayload::Integer(val as i32);
                        v.size_len = if val as i32 == BUSTUB_INT32_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    TypeId::Timestamp => {
                        v.raw_value = ValuePayload::Timestamp(val as u64);
                        v.size_len = if val as u64 == BUSTUB_TIMESTAMP_NULL {
                            BUSTUB_VALUE_NULL
                        } else {
                            0
                        };
                    }
                    _ => {
                        panic!("Invalid Type for i64 Value constructor");
                    }
                }
            }
        }
        v
    }

    /// Create a timestamp value from u64.
    pub fn from_u64(type_id: TypeId, val: u64) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Timestamp => {
                v.raw_value = ValuePayload::Timestamp(val);
                v.size_len = if val == BUSTUB_TIMESTAMP_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            _ => {
                panic!("Invalid Type for u64 Value constructor");
            }
        }
        v
    }

    /// Create a decimal value.
    pub fn from_f64(type_id: TypeId, val: f64) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Decimal => {
                v.raw_value = ValuePayload::Decimal(val);
                v.size_len = if val == BUSTUB_DECIMAL_NULL {
                    BUSTUB_VALUE_NULL
                } else {
                    0
                };
            }
            _ => {
                panic!("Invalid Type for f64 Value constructor");
            }
        }
        v
    }

    /// Create a VARCHAR value from a byte slice.
    pub fn from_bytes(type_id: TypeId, data: &[u8], len: u32, manage_data: bool) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Varchar | TypeId::Vector => {
                if data.is_empty() {
                    v.raw_value = ValuePayload::VarlenRef;
                    v.size_len = BUSTUB_VALUE_NULL;
                } else {
                    v.manage_data = manage_data;
                    if manage_data {
                        assert!(len < BUSTUB_VARCHAR_MAX_LEN);
                        let owned = data[..len as usize].to_vec();
                        v.raw_value = ValuePayload::Varlen(owned);
                        v.size_len = len;
                    } else {
                        v.raw_value = ValuePayload::Varlen(data.to_vec());
                        v.size_len = len;
                    }
                }
            }
            _ => {
                panic!("Invalid Type for variable-length Value constructor");
            }
        }
        v
    }

    /// Create a VARCHAR value from a string.
    pub fn from_string(type_id: TypeId, data: &str) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Varchar => {
                v.manage_data = true;
                let len = data.len() as u32 + 1; // +1 for null terminator
                let mut owned = Vec::with_capacity(len as usize);
                owned.extend_from_slice(data.as_bytes());
                owned.push(0); // null-terminated
                v.raw_value = ValuePayload::Varlen(owned);
                v.size_len = len;
            }
            _ => {
                panic!("Invalid Type for string Value constructor");
            }
        }
        v
    }

    /// Create a VECTOR value from a slice of doubles.
    pub fn from_vec_double(type_id: TypeId, data: &[f64]) -> Self {
        let mut v = Value::new(type_id);
        match type_id {
            TypeId::Vector => {
                v.manage_data = true;
                let len = data.len() * 8; // 8 bytes per double
                let mut owned = Vec::with_capacity(len);
                for d in data {
                    owned.extend_from_slice(&d.to_le_bytes());
                }
                v.raw_value = ValuePayload::Varlen(owned);
                v.size_len = len as u32;
            }
            _ => {
                panic!("Invalid Type for vector Value constructor");
            }
        }
        v
    }

    // --- Accessors ---

    /// Get the type ID of this value.
    pub fn get_type_id(&self) -> TypeId {
        self.sql_type_id
    }

    /// Get the storage size of this value.
    pub fn get_storage_size(&self) -> u32 {
        get_type_instance(self.sql_type_id).get_storage_size(self)
    }

    /// Access the raw variable-length data.
    pub fn get_data(&self) -> &[u8] {
        get_type_instance(self.sql_type_id).get_data(self)
    }

    /// Get the raw value as a specific type.
    pub fn get_as<T>(&self) -> T
    where
        T: Copy + 'static,
    {
        unsafe {
            match self.raw_value {
                ValuePayload::Boolean(v) => *(&v as *const i8 as *const T),
                ValuePayload::TinyInt(v) => *(&v as *const i8 as *const T),
                ValuePayload::SmallInt(v) => *(&v as *const i16 as *const T),
                ValuePayload::Integer(v) => *(&v as *const i32 as *const T),
                ValuePayload::BigInt(v) => *(&v as *const i64 as *const T),
                ValuePayload::Decimal(v) => *(&v as *const f64 as *const T),
                ValuePayload::Timestamp(v) => *(&v as *const u64 as *const T),
                ValuePayload::Nil | ValuePayload::VarlenRef | ValuePayload::Varlen(_) => {
                    panic!("Cannot get raw value from variable-length type")
                }
            }
        }
    }

    /// Get the vector data for VECTOR type.
    pub fn get_vector(&self) -> Vec<f64> {
        match &self.raw_value {
            ValuePayload::Varlen(data) => {
                let elem_count = data.len() / 8;
                let mut result = Vec::with_capacity(elem_count);
                for i in 0..elem_count {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[i * 8..(i + 1) * 8]);
                    result.push(f64::from_le_bytes(bytes));
                }
                result
            }
            _ => panic!("Not a vector value"),
        }
    }

    /// Check if this value is an integer type.
    pub fn check_integer(&self) -> bool {
        matches!(
            self.sql_type_id,
            TypeId::TinyInt | TypeId::SmallInt | TypeId::Integer | TypeId::BigInt
        )
    }

    /// Check if this value is comparable with the other value.
    pub fn check_comparable(&self, other: &Value) -> bool {
        match self.sql_type_id {
            TypeId::Boolean => matches!(other.sql_type_id, TypeId::Boolean | TypeId::Varchar),
            TypeId::TinyInt | TypeId::SmallInt | TypeId::Integer | TypeId::BigInt | TypeId::Decimal => {
                matches!(
                    other.sql_type_id,
                    TypeId::TinyInt
                        | TypeId::SmallInt
                        | TypeId::Integer
                        | TypeId::BigInt
                        | TypeId::Decimal
                        | TypeId::Varchar
                )
            }
            TypeId::Varchar => true, // Anything can be cast to a string!
            _ => false,
        }
    }

    /// Check if this value is NULL.
    pub fn is_null(&self) -> bool {
        self.size_len == BUSTUB_VALUE_NULL
    }

    // --- Casting ---
    pub fn cast_as(&self, type_id: TypeId) -> Value {
        get_type_instance(self.sql_type_id).cast_as(self, type_id)
    }

    // --- Comparison Methods ---
    pub fn compare_equals(&self, other: &Value) -> CmpBool {
        get_type_instance(self.sql_type_id).compare_equals(self, other)
    }

    pub fn compare_not_equals(&self, other: &Value) -> CmpBool {
        get_type_instance(self.sql_type_id).compare_not_equals(self, other)
    }

    pub fn compare_less_than(&self, other: &Value) -> CmpBool {
        get_type_instance(self.sql_type_id).compare_less_than(self, other)
    }

    pub fn compare_less_than_equals(&self, other: &Value) -> CmpBool {
        get_type_instance(self.sql_type_id).compare_less_than_equals(self, other)
    }

    pub fn compare_greater_than(&self, other: &Value) -> CmpBool {
        get_type_instance(self.sql_type_id).compare_greater_than(self, other)
    }

    pub fn compare_greater_than_equals(&self, other: &Value) -> CmpBool {
        get_type_instance(self.sql_type_id).compare_greater_than_equals(self, other)
    }

    /// Check exact equality (with NULL = NULL = true semantics).
    pub fn compare_exactly_equals(&self, other: &Value) -> bool {
        if self.is_null() && other.is_null() {
            return true;
        }
        get_type_instance(self.sql_type_id).compare_equals(self, other) == CmpBool::CmpTrue
    }

    // --- Arithmetic Methods ---
    pub fn add(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).add(self, other)
    }

    pub fn subtract(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).subtract(self, other)
    }

    pub fn multiply(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).multiply(self, other)
    }

    pub fn divide(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).divide(self, other)
    }

    pub fn modulo(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).modulo(self, other)
    }

    pub fn min_val(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).min_val(self, other)
    }

    pub fn max_val(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).max_val(self, other)
    }

    pub fn sqrt(&self) -> Value {
        get_type_instance(self.sql_type_id).sqrt(self)
    }

    pub fn operate_null(&self, other: &Value) -> Value {
        get_type_instance(self.sql_type_id).operate_null(self, other)
    }

    pub fn is_zero(&self) -> bool {
        get_type_instance(self.sql_type_id).is_zero(self)
    }

    // --- Serialization ---
    pub fn serialize_to(&self, storage: &mut [u8]) {
        get_type_instance(self.sql_type_id).serialize_to(self, storage);
    }

    pub fn deserialize_from(storage: &[u8], type_id: TypeId) -> Value {
        get_type_instance(type_id).deserialize_from(storage)
    }

    /// Return a string version of this value.
    pub fn to_string_val(&self) -> String {
        get_type_instance(self.sql_type_id).to_string_val(self)
    }

    /// Create a copy of this value.
    pub fn copy_val(&self) -> Value {
        get_type_instance(self.sql_type_id).copy(self)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_val())
    }
}


