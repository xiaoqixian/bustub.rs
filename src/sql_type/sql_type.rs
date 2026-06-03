//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// sql_type.rs
//
// Identification: src/sql_type/sql_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::bigint_type::BigintType;
use super::boolean_type::BooleanType;
use super::decimal_type::DecimalType;
use super::integer_type::IntegerType;
use super::limits::*;
use super::smallint_type::SmallintType;
use super::timestamp_type::TimestampType;
use super::tinyint_type::TinyintType;
use super::type_id::TypeId;
use super::value::Value;
use super::varlen_type::VarlenType;
use super::vector_type::VectorType;
use std::fmt;
use std::sync::OnceLock;

/// Comparison result enum that supports three-valued logic (SQL NULL semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpBool {
    CmpFalse = 0,
    CmpTrue = 1,
    CmpNull = 2,
}

/// Helper to convert a boolean to a CmpBool.
pub fn get_cmp_bool(boolean: bool) -> CmpBool {
    if boolean {
        CmpBool::CmpTrue
    } else {
        CmpBool::CmpFalse
    }
}

impl fmt::Display for CmpBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CmpBool::CmpFalse => write!(f, "false"),
            CmpBool::CmpTrue => write!(f, "true"),
            CmpBool::CmpNull => write!(f, "null"),
        }
    }
}

/// Convert a TypeId to its string representation.
pub fn type_id_to_string(type_id: TypeId) -> String {
    match type_id {
        TypeId::Boolean => "BOOLEAN".to_string(),
        TypeId::TinyInt => "TINYINT".to_string(),
        TypeId::SmallInt => "SMALLINT".to_string(),
        TypeId::Integer => "INTEGER".to_string(),
        TypeId::BigInt => "BIGINT".to_string(),
        TypeId::Decimal => "DECIMAL".to_string(),
        TypeId::Timestamp => "TIMESTAMP".to_string(),
        TypeId::Varchar => "VARCHAR".to_string(),
    }
}

/// Get the size of a data type in bytes.
pub fn get_type_size(type_id: TypeId) -> u64 {
    match type_id {
        TypeId::Boolean | TypeId::TinyInt => 1,
        TypeId::SmallInt => 2,
        TypeId::Integer => 4,
        TypeId::BigInt | TypeId::Decimal | TypeId::Timestamp => 8,
        TypeId::Varchar => 0,
    }
}

/// Get the minimum value for a given type.
pub fn get_min_value(type_id: TypeId) -> Value {
    match type_id {
        TypeId::Boolean => Value::from_i8(TypeId::Boolean, 0),
        TypeId::TinyInt => Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_MIN),
        TypeId::SmallInt => Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_MIN),
        TypeId::Integer => Value::from_i32(TypeId::Integer, BUSTUB_INT32_MIN),
        TypeId::BigInt => Value::from_i64(TypeId::BigInt, BUSTUB_INT64_MIN),
        TypeId::Decimal => Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_MIN),
        TypeId::Timestamp => Value::from_u64(TypeId::Timestamp, 0),
        TypeId::Varchar => Value::from_string(TypeId::Varchar, ""),
    }
}

/// Get the maximum value for a given type.
pub fn get_max_value(type_id: TypeId) -> Value {
    match type_id {
        TypeId::Boolean => Value::from_i8(TypeId::Boolean, 1),
        TypeId::TinyInt => Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_MAX),
        TypeId::SmallInt => Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_MAX),
        TypeId::Integer => Value::from_i32(TypeId::Integer, BUSTUB_INT32_MAX),
        TypeId::BigInt => Value::from_i64(TypeId::BigInt, BUSTUB_INT64_MAX),
        TypeId::Decimal => Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_MAX),
        TypeId::Timestamp => Value::from_u64(TypeId::Timestamp, BUSTUB_TIMESTAMP_MAX),
        TypeId::Varchar => Value::from_bytes(TypeId::Varchar, &[], 0, false),
    }
}

/// Get the singleton instance for a given type.
pub fn get_type_instance(type_id: TypeId) -> &'static dyn SqlType {
    static INSTANCES: OnceLock<Vec<Box<dyn SqlType>>> = OnceLock::new();
    let instances = INSTANCES.get_or_init(|| {
        vec![
            // Index 1: BOOLEAN
            Box::new(BooleanType::new()),
            // Index 2: TINYINT
            Box::new(TinyintType::new()),
            // Index 3: SMALLINT
            Box::new(SmallintType::new()),
            // Index 4: INTEGER
            Box::new(IntegerType::new()),
            // Index 5: BIGINT
            Box::new(BigintType::new()),
            // Index 6: DECIMAL
            Box::new(DecimalType::new()),
            // Index 7: VARCHAR
            Box::new(VarlenType::new(TypeId::Varchar)),
            // Index 8: TIMESTAMP
            Box::new(TimestampType::new()),
            // Index 9: VECTOR
            Box::new(VectorType::new()),
        ]
    });
    let idx = type_id as usize;
    instances[idx].as_ref()
}

/// The base trait for all SQL types.
///
/// All SQL type implementations should implement this trait to provide
/// comparison, arithmetic, serialization, and type conversion operations.
pub trait SqlType: Send + Sync {
    /// Returns the TypeId of this type.
    fn get_type_id(&self) -> TypeId;

    /// Get the size of this data type in bytes.
    fn get_type_size(&self) -> u64;

    /// Is this type coercable from the other type.
    fn is_coercable_from(&self, type_id: TypeId) -> bool;

    /// Convert this type's ID to its string representation.
    fn to_string_id(&self) -> String;

    /// Get the minimum value for this type.
    fn get_min_value(&self) -> Value;

    /// Get the maximum value for this type.
    fn get_max_value(&self) -> Value;

    // --- Comparison functions ---
    fn compare_equals(&self, left: &Value, right: &Value) -> CmpBool;
    fn compare_not_equals(&self, left: &Value, right: &Value) -> CmpBool;
    fn compare_less_than(&self, left: &Value, right: &Value) -> CmpBool;
    fn compare_less_than_equals(&self, left: &Value, right: &Value) -> CmpBool;
    fn compare_greater_than(&self, left: &Value, right: &Value) -> CmpBool;
    fn compare_greater_than_equals(&self, left: &Value, right: &Value) -> CmpBool;

    // --- Arithmetic functions ---
    fn add(&self, left: &Value, right: &Value) -> Value;
    fn subtract(&self, left: &Value, right: &Value) -> Value;
    fn multiply(&self, left: &Value, right: &Value) -> Value;
    fn divide(&self, left: &Value, right: &Value) -> Value;
    fn modulo(&self, left: &Value, right: &Value) -> Value;
    fn min_val(&self, left: &Value, right: &Value) -> Value;
    fn max_val(&self, left: &Value, right: &Value) -> Value;
    fn sqrt(&self, val: &Value) -> Value;
    fn operate_null(&self, left: &Value, right: &Value) -> Value;
    fn is_zero(&self, val: &Value) -> bool;

    /// Is the data inlined into this type's storage space.
    fn is_inlined(&self, val: &Value) -> bool;

    /// Return a stringified version of this value.
    fn to_string_val(&self, val: &Value) -> String;

    /// Serialize this value into the given storage space.
    fn serialize_to(&self, val: &Value, storage: &mut [u8]);

    /// Deserialize a value of this type from the given storage space.
    fn deserialize_from(&self, storage: &[u8]) -> Value;

    /// Create a copy of this value.
    fn copy(&self, val: &Value) -> Value;

    /// Cast this value to another type.
    fn cast_as(&self, val: &Value, type_id: TypeId) -> Value;

    /// Access the raw variable-length data stored in the tuple storage.
    fn get_data<'a>(&self, val: &'a Value) -> &'a [u8];

    /// Get the storage size of the value.
    fn get_storage_size(&self, val: &Value) -> u32;
}


