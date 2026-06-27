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
use super::TypeId;
use super::CmpBool;
use std::cmp::Ordering;

/// Compute the modulo of x and y for floating-point values.
fn val_mod(x: f64, y: f64) -> f64 {
    x - (x / y).trunc() * y
}

fn compare_strings(str1: &[u8], len1: usize, str2: &[u8], len2: usize) -> Ordering {
    let min_len = len1.min(len2);
    let a = &str1[..min_len];
    let b = &str2[..min_len];
    match a.cmp(b) {
        Ordering::Equal => {
            if len1 != len2 {
                len1.cmp(&len2)
            } else {
                Ordering::Equal
            }
        }
        other => other,
    }
}

pub enum Value {
    Boolean(i8),
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Decimal(f64),
    Timestamp(u64),
    Varlen(Vec<u8>),
    /// For VARCHAR values where data is not owned by this Value.
    VarlenRef((*const u8, usize)),
    /// For invalid/null typed values.
    Nil,
}

impl Value {
    // ──────────────────────────────────────────────
    //  Constructors
    // ──────────────────────────────────────────────

    /// Create a new Value as NULL.
    pub fn new(_type_id: TypeId) -> Self {
        Value::Nil
    }

    pub fn from_bool(val: bool) -> Self {
        match val {
            true => Value::Boolean(1),
            false => Value::Boolean(0),
        }
    }

    pub fn from_i8(val: i8) -> Self {
        match val {
            BUSTUB_INT8_NULL => Value::Nil,
            _ => Value::TinyInt(val)
        }
    }

    /// Create a smallint value.
    pub fn from_i16(val: i16) -> Self {
        match val {
            BUSTUB_INT16_NULL => Value::Nil,
            _ => Value::SmallInt(val)
        }
    }

    /// Create an integer value.
    pub fn from_i32(val: i32) -> Self {
        match val {
            BUSTUB_INT32_NULL => Value::Nil,
            _ => Value::Integer(val)
        }
    }

    /// Create a bigint or timestamp value. Also supports constructing other
    /// types via truncation of the i64 value.
    pub fn from_i64(val: i64) -> Self {
        match val {
            BUSTUB_INT64_NULL => Value::Nil,
            _ => Value::BigInt(val)
        }
    }

    /// Create a timestamp value from u64.
    pub fn from_u64(val: u64) -> Self {
        match val {
            BUSTUB_TIMESTAMP_NULL => Value::Nil,
            _ => Value::Timestamp(val)
        }
    }

    /// Create a decimal value.
    pub fn from_f64(val: f64) -> Self {
        match val {
            BUSTUB_DECIMAL_NULL => Value::Nil,
            _ => Value::Decimal(val)
        }
    }

    /// Create a VARCHAR value from a byte slice.
    ///
    /// If `manage_data` is true, the data is copied into an owned `Vec<u8>`.
    /// If `manage_data` is false, a raw pointer + length is stored (the caller
    /// must ensure the data outlives this `Value`).
    pub fn from_bytes(data: &[u8], len: u32, manage_data: bool) -> Self {
        if data.is_empty() {
            return Value::Nil;
        }
        if manage_data {
            assert!(len < BUSTUB_VARCHAR_MAX_LEN);
            let owned = data[..len as usize].to_vec();
            Value::Varlen(owned)
        } else {
            // Caller guarantees the data will outlive this Value.
            Value::VarlenRef((data.as_ptr(), len as usize))
        }
    }

    /// Create a VARCHAR value from a string (null-terminated).
    pub fn from_str(data: &str) -> Self {
        let len = data.len() + 1; // +1 for null terminator
        let mut owned = Vec::with_capacity(len);
        owned.extend_from_slice(data.as_bytes());
        owned.push(0);
        Value::Varlen(owned)
    }

    pub fn from_cmp_bool(val: CmpBool) -> Self {
        match val {
            CmpBool::CmpTrue => Value::Boolean(1),
            CmpBool::CmpFalse => Value::Boolean(0),
            CmpBool::CmpNull => Value::Boolean(BUSTUB_BOOLEAN_NULL),
        }
    }

    /// Create a zero value of the given type.
    pub fn zero(type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => Value::from_i8(0),
            TypeId::TinyInt => Value::from_i8(0),
            TypeId::SmallInt => Value::from_i16(0),
            TypeId::Integer => Value::from_i32(0),
            TypeId::BigInt => Value::from_i64(0),
            TypeId::Decimal => Value::from_f64(0.0),
            TypeId::Varchar => Value::from_str("0"),
            _ => panic!("Unknown type for get_zero_value_by_type"),
        }
    }

    pub fn null(type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => Value::from_i8(BUSTUB_BOOLEAN_NULL),
            TypeId::TinyInt => Value::from_i8(BUSTUB_INT8_NULL),
            TypeId::SmallInt => Value::from_i16(BUSTUB_INT16_NULL),
            TypeId::Integer => Value::from_i32(BUSTUB_INT32_NULL),
            TypeId::BigInt => Value::from_i64(BUSTUB_INT64_NULL),
            TypeId::Decimal => Value::from_f64(BUSTUB_DECIMAL_NULL),
            TypeId::Varchar => Value::from_bytes(&[], 0, false),
            _ => panic!("Attempting to create invalid null type"),
        }
    }

    // ──────────────────────────────────────────────
    //  Accessors
    // ──────────────────────────────────────────────

    /// Get the type ID of this value.
    pub fn get_type_id(&self) -> TypeId {
        match self {
            Value::Boolean(_) => TypeId::Boolean,
            Value::TinyInt(_) => TypeId::TinyInt,
            Value::SmallInt(_) => TypeId::SmallInt,
            Value::Integer(_) => TypeId::Integer,
            Value::BigInt(_) => TypeId::BigInt,
            Value::Decimal(_) => TypeId::Decimal,
            Value::Timestamp(_) => TypeId::Timestamp,
            Value::Varlen(_) | Value::VarlenRef(_) => TypeId::Varchar,
            Value::Nil => panic!("Cannot get type id from nil value"),
        }
    }

    /// Get the storage size of this value.
    pub fn get_storage_size(&self) -> usize {
        match self {
            Value::Nil => 0,
            Value::Boolean(_) | Value::TinyInt(_) => 1,
            Value::SmallInt(_) => 2,
            Value::Integer(_) => 4,
            Value::BigInt(_) | Value::Decimal(_) | Value::Timestamp(_) => 8,
            Value::Varlen(data) => data.len(),
            Value::VarlenRef((_, len)) => *len,
        }
    }

    /// Access the raw variable-length data (for VARCHAR types).
    pub fn get_data(&self) -> &[u8] {
        match self {
            Value::Varlen(data) => data.as_slice(),
            Value::VarlenRef((ptr, len)) => unsafe { std::slice::from_raw_parts(*ptr, *len) },
            _ => panic!("GetData not implemented for this type"),
        }
    }

    /// Get the raw value as a specific type via raw memory reinterpretation.
    /// Only works for fixed-size types (Boolean, TinyInt, SmallInt, Integer,
    /// BigInt, Decimal, Timestamp).
    pub fn get_as<T>(&self) -> T
    where
        T: Copy + 'static,
    {
        unsafe {
            match self {
                Value::Boolean(v) => *(v as *const i8 as *const T),
                Value::TinyInt(v) => *(v as *const i8 as *const T),
                Value::SmallInt(v) => *(v as *const i16 as *const T),
                Value::Integer(v) => *(v as *const i32 as *const T),
                Value::BigInt(v) => *(v as *const i64 as *const T),
                Value::Decimal(v) => *(v as *const f64 as *const T),
                Value::Timestamp(v) => *(v as *const u64 as *const T),
                Value::Nil | Value::VarlenRef(_) | Value::Varlen(_) => {
                    panic!("Cannot get raw value from variable-length or nil type")
                }
            }
        }
    }

    /// Get the vector data (for VECTOR type, stored in Varlen).
    pub fn get_vector(&self) -> Vec<f64> {
        match self {
            Value::Varlen(data) => {
                let elem_count = data.len() / 8;
                let mut result = Vec::with_capacity(elem_count);
                for i in 0..elem_count {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[i * 8..(i + 1) * 8]);
                    result.push(f64::from_le_bytes(bytes));
                }
                result
            }
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
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
            self,
            Value::TinyInt(_) | Value::SmallInt(_) | Value::Integer(_) | Value::BigInt(_)
        )
    }

    /// Check if this value is comparable with the other value.
    pub fn check_comparable(&self, other: &Value) -> bool {
        match self {
            Value::Boolean(_) => {
                matches!(other, Value::Boolean(_) | Value::Varlen(_) | Value::VarlenRef(_))
            }
            Value::TinyInt(_)
            | Value::SmallInt(_)
            | Value::Integer(_)
            | Value::BigInt(_)
            | Value::Decimal(_) => {
                matches!(
                    other,
                    Value::TinyInt(_)
                        | Value::SmallInt(_)
                        | Value::Integer(_)
                        | Value::BigInt(_)
                        | Value::Decimal(_)
                        | Value::Varlen(_)
                        | Value::VarlenRef(_)
                )
            }
            Value::Timestamp(_) => {
                matches!(
                    other,
                    Value::Timestamp(_) | Value::Varlen(_) | Value::VarlenRef(_)
                )
            }
            Value::Varlen(_) | Value::VarlenRef(_) => true, // Anything can be cast to a string!
            Value::Nil => false,
        }
    }

    /// Check if this value is NULL.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Nil)
    }

    // ──────────────────────────────────────────────
    //  Casting
    // ──────────────────────────────────────────────

    pub fn cast_as(&self, type_id: TypeId) -> Value {
        match self {
            Value::Nil => match type_id {
                TypeId::Varchar => Value::from_bytes(&[], 0, false),
                _ => Value::Nil,
            },
            Value::Boolean(v) => Value::cast_boolean(*v, type_id),
            Value::TinyInt(v) => Value::cast_tinyint(*v, type_id),
            Value::SmallInt(v) => Value::cast_smallint(*v, type_id),
            Value::Integer(v) => Value::cast_integer(*v, type_id),
            Value::BigInt(v) => Value::cast_bigint(*v, type_id),
            Value::Decimal(v) => Value::cast_decimal(*v, type_id),
            Value::Timestamp(v) => Value::cast_timestamp(*v, type_id),
            Value::Varlen(data) => Value::cast_varlen_str(
                &data[..data.len().saturating_sub(1)],
                type_id,
            ),
            Value::VarlenRef((ptr, len)) => {
                let s = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                Value::cast_varlen_str(&s[..s.len().saturating_sub(1)], type_id)
            }
        }
    }

    fn cast_boolean(v: i8, type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => Value::Boolean(v),
            TypeId::Varchar => Value::from_str(if v == 1 { "true" } else { "false" }),
            _ => {
                panic!("BOOLEAN is not coercable to {}", type_id_to_string(type_id))
            }
        }
    }

    fn cast_tinyint(v: i8, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => Value::TinyInt(v),
            TypeId::SmallInt => Value::SmallInt(v as i16),
            TypeId::Integer => Value::Integer(v as i32),
            TypeId::BigInt => Value::BigInt(v as i64),
            TypeId::Decimal => Value::Decimal(v as f64),
            TypeId::Varchar => Value::from_str(&v.to_string()),
            _ => {
                panic!("TINYINT is not coercable to {}", type_id_to_string(type_id))
            }
        }
    }

    fn cast_smallint(v: i16, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => {
                if v > BUSTUB_INT8_MAX as i16 || v < BUSTUB_INT8_MIN as i16 {
                    panic!("Numeric value out of range.");
                }
                Value::TinyInt(v as i8)
            }
            TypeId::SmallInt => Value::SmallInt(v),
            TypeId::Integer => Value::Integer(v as i32),
            TypeId::BigInt => Value::BigInt(v as i64),
            TypeId::Decimal => Value::Decimal(v as f64),
            TypeId::Varchar => Value::from_str(&v.to_string()),
            _ => {
                panic!("SMALLINT is not coercable to {}", type_id_to_string(type_id))
            }
        }
    }

    fn cast_integer(v: i32, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => {
                if v > BUSTUB_INT8_MAX as i32 || v < BUSTUB_INT8_MIN as i32 {
                    panic!("Numeric value out of range.");
                }
                Value::TinyInt(v as i8)
            }
            TypeId::SmallInt => {
                if v > BUSTUB_INT16_MAX as i32 || v < BUSTUB_INT16_MIN as i32 {
                    panic!("Numeric value out of range.");
                }
                Value::SmallInt(v as i16)
            }
            TypeId::Integer => Value::Integer(v),
            TypeId::BigInt => Value::BigInt(v as i64),
            TypeId::Decimal => Value::Decimal(v as f64),
            TypeId::Varchar => Value::from_str(&v.to_string()),
            _ => {
                panic!("INTEGER is not coercable to {}", type_id_to_string(type_id))
            }
        }
    }

    fn cast_bigint(v: i64, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => {
                if v > BUSTUB_INT8_MAX as i64 || v < BUSTUB_INT8_MIN as i64 {
                    panic!("Numeric value out of range.");
                }
                Value::TinyInt(v as i8)
            }
            TypeId::SmallInt => {
                if v > BUSTUB_INT16_MAX as i64 || v < BUSTUB_INT16_MIN as i64 {
                    panic!("Numeric value out of range.");
                }
                Value::SmallInt(v as i16)
            }
            TypeId::Integer => {
                if v > BUSTUB_INT32_MAX as i64 || v < BUSTUB_INT32_MIN as i64 {
                    panic!("Numeric value out of range.");
                }
                Value::Integer(v as i32)
            }
            TypeId::BigInt => Value::BigInt(v),
            TypeId::Decimal => Value::Decimal(v as f64),
            TypeId::Varchar => Value::from_str(&v.to_string()),
            _ => {
                panic!("BIGINT is not coercable to {}", type_id_to_string(type_id))
            }
        }
    }

    fn cast_decimal(v: f64, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => {
                if v > BUSTUB_INT8_MAX as f64 || v < BUSTUB_INT8_MIN as f64 {
                    panic!("Numeric value out of range.");
                }
                Value::TinyInt(v as i8)
            }
            TypeId::SmallInt => {
                if v > BUSTUB_INT16_MAX as f64 || v < BUSTUB_INT16_MIN as f64 {
                    panic!("Numeric value out of range.");
                }
                Value::SmallInt(v as i16)
            }
            TypeId::Integer => {
                if v > BUSTUB_INT32_MAX as f64 || v < BUSTUB_INT32_MIN as f64 {
                    panic!("Numeric value out of range.");
                }
                Value::Integer(v as i32)
            }
            TypeId::BigInt => {
                if v >= BUSTUB_INT64_MAX as f64 || v < BUSTUB_INT64_MIN as f64 {
                    panic!("Numeric value out of range.");
                }
                Value::BigInt(v as i64)
            }
            TypeId::Decimal => Value::Decimal(v),
            TypeId::Varchar => Value::from_str(&v.to_string()),
            _ => {
                panic!("DECIMAL is not coercable to {}", type_id_to_string(type_id))
            }
        }
    }

    fn cast_timestamp(v: u64, type_id: TypeId) -> Value {
        match type_id {
            TypeId::Timestamp => Value::Timestamp(v),
            TypeId::Varchar => Value::from_str(&format_timestamp(v)),
            _ => {
                panic!(
                    "TIMESTAMP is not coercable to {}",
                    type_id_to_string(type_id)
                )
            }
        }
    }

    fn cast_varlen_str(s: &[u8], type_id: TypeId) -> Value {
        let str_val = String::from_utf8_lossy(s);
        match type_id {
            TypeId::Boolean => {
                let lower = str_val.to_lowercase();
                if lower == "true" || lower == "1" || lower == "t" {
                    return Value::Boolean(1);
                }
                if lower == "false" || lower == "0" || lower == "f" {
                    return Value::Boolean(0);
                }
                panic!("Boolean value format error.");
            }
            TypeId::TinyInt => {
                let v: i8 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                Value::TinyInt(v)
            }
            TypeId::SmallInt => {
                let v: i16 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                Value::SmallInt(v)
            }
            TypeId::Integer => {
                let v: i32 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                Value::Integer(v)
            }
            TypeId::BigInt => {
                let v: i64 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                Value::BigInt(v)
            }
            TypeId::Decimal => {
                let v: f64 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                Value::Decimal(v)
            }
            TypeId::Varchar => Value::from_str(&str_val),
            _ => {
                panic!(
                    "VARCHAR is not coercable to {}",
                    type_id_to_string(type_id)
                )
            }
        }
    }

    // ──────────────────────────────────────────────
    //  Comparison Methods
    // ──────────────────────────────────────────────

    pub fn compare_equals(&self, other: &Value) -> CmpBool {
        if self.is_null() || other.is_null() {
            return CmpBool::CmpNull;
        }
        match self {
            Value::Boolean(v) => compare_boolean(*v, other, |a, b| a == b),
            Value::TinyInt(v) => compare_tinyint(*v as i64, other, |a, b| a == b),
            Value::SmallInt(v) => compare_smallint(*v as i64, other, |a, b| a == b),
            Value::Integer(v) => compare_integer(*v as i64, other, |a, b| a == b),
            Value::BigInt(v) => compare_bigint(*v, other, |a, b| a == b),
            Value::Decimal(v) => compare_decimal(*v, other, |a, b| (a - b).abs() < f64::EPSILON),
            Value::Timestamp(v) => compare_timestamp(*v, other, |a, b| a == b),
            Value::Varlen(data) => compare_varlen_with_len(
                data.as_slice(),
                data.len(),
                other,
                |ord| ord == Ordering::Equal,
            ),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                compare_varlen_with_len(data, *len, other, |ord| ord == Ordering::Equal)
            }
            Value::Nil => CmpBool::CmpNull,
        }
    }

    pub fn compare_not_equals(&self, other: &Value) -> CmpBool {
        if self.is_null() || other.is_null() {
            return CmpBool::CmpNull;
        }
        match self {
            Value::Boolean(v) => compare_boolean(*v, other, |a, b| a != b),
            Value::TinyInt(v) => compare_tinyint(*v as i64, other, |a, b| a != b),
            Value::SmallInt(v) => compare_smallint(*v as i64, other, |a, b| a != b),
            Value::Integer(v) => compare_integer(*v as i64, other, |a, b| a != b),
            Value::BigInt(v) => compare_bigint(*v, other, |a, b| a != b),
            Value::Decimal(v) => compare_decimal(*v, other, |a, b| (a - b).abs() >= f64::EPSILON),
            Value::Timestamp(v) => compare_timestamp(*v, other, |a, b| a != b),
            Value::Varlen(data) => compare_varlen_with_len(
                data.as_slice(),
                data.len(),
                other,
                |ord| ord != Ordering::Equal,
            ),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                compare_varlen_with_len(data, *len, other, |ord| ord != Ordering::Equal)
            }
            Value::Nil => CmpBool::CmpNull,
        }
    }

    pub fn compare_less_than(&self, other: &Value) -> CmpBool {
        if self.is_null() || other.is_null() {
            return CmpBool::CmpNull;
        }
        match self {
            Value::Boolean(v) => compare_boolean(*v, other, |a, b| a < b),
            Value::TinyInt(v) => compare_tinyint(*v as i64, other, |a, b| a < b),
            Value::SmallInt(v) => compare_smallint(*v as i64, other, |a, b| a < b),
            Value::Integer(v) => compare_integer(*v as i64, other, |a, b| a < b),
            Value::BigInt(v) => compare_bigint(*v, other, |a, b| a < b),
            Value::Decimal(v) => compare_decimal(*v, other, |a, b| a < b),
            Value::Timestamp(v) => compare_timestamp(*v, other, |a, b| a < b),
            Value::Varlen(data) => compare_varlen_with_len(
                data.as_slice(),
                data.len(),
                other,
                |ord| ord == Ordering::Less,
            ),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                compare_varlen_with_len(data, *len, other, |ord| ord == Ordering::Less)
            }
            Value::Nil => CmpBool::CmpNull,
        }
    }

    pub fn compare_less_than_equals(&self, other: &Value) -> CmpBool {
        if self.is_null() || other.is_null() {
            return CmpBool::CmpNull;
        }
        match self {
            Value::Boolean(v) => compare_boolean(*v, other, |a, b| a <= b),
            Value::TinyInt(v) => compare_tinyint(*v as i64, other, |a, b| a <= b),
            Value::SmallInt(v) => compare_smallint(*v as i64, other, |a, b| a <= b),
            Value::Integer(v) => compare_integer(*v as i64, other, |a, b| a <= b),
            Value::BigInt(v) => compare_bigint(*v, other, |a, b| a <= b),
            Value::Decimal(v) => compare_decimal(*v, other, |a, b| a <= b),
            Value::Timestamp(v) => compare_timestamp(*v, other, |a, b| a <= b),
            Value::Varlen(data) => compare_varlen_with_len(
                data.as_slice(),
                data.len(),
                other,
                |ord| ord != Ordering::Greater,
            ),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                compare_varlen_with_len(data, *len, other, |ord| ord != Ordering::Greater)
            }
            Value::Nil => CmpBool::CmpNull,
        }
    }

    pub fn compare_greater_than(&self, other: &Value) -> CmpBool {
        if self.is_null() || other.is_null() {
            return CmpBool::CmpNull;
        }
        match self {
            Value::Boolean(v) => compare_boolean(*v, other, |a, b| a > b),
            Value::TinyInt(v) => compare_tinyint(*v as i64, other, |a, b| a > b),
            Value::SmallInt(v) => compare_smallint(*v as i64, other, |a, b| a > b),
            Value::Integer(v) => compare_integer(*v as i64, other, |a, b| a > b),
            Value::BigInt(v) => compare_bigint(*v, other, |a, b| a > b),
            Value::Decimal(v) => compare_decimal(*v, other, |a, b| a > b),
            Value::Timestamp(v) => compare_timestamp(*v, other, |a, b| a > b),
            Value::Varlen(data) => compare_varlen_with_len(
                data.as_slice(),
                data.len(),
                other,
                |ord| ord == Ordering::Greater,
            ),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                compare_varlen_with_len(data, *len, other, |ord| ord == Ordering::Greater)
            }
            Value::Nil => CmpBool::CmpNull,
        }
    }

    pub fn compare_greater_than_equals(&self, other: &Value) -> CmpBool {
        if self.is_null() || other.is_null() {
            return CmpBool::CmpNull;
        }
        match self {
            Value::Boolean(v) => compare_boolean(*v, other, |a, b| a >= b),
            Value::TinyInt(v) => compare_tinyint(*v as i64, other, |a, b| a >= b),
            Value::SmallInt(v) => compare_smallint(*v as i64, other, |a, b| a >= b),
            Value::Integer(v) => compare_integer(*v as i64, other, |a, b| a >= b),
            Value::BigInt(v) => compare_bigint(*v, other, |a, b| a >= b),
            Value::Decimal(v) => compare_decimal(*v, other, |a, b| a >= b),
            Value::Timestamp(v) => compare_timestamp(*v, other, |a, b| a >= b),
            Value::Varlen(data) => compare_varlen_with_len(
                data.as_slice(),
                data.len(),
                other,
                |ord| ord != Ordering::Less,
            ),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                compare_varlen_with_len(data, *len, other, |ord| ord != Ordering::Less)
            }
            Value::Nil => CmpBool::CmpNull,
        }
    }

    /// Check exact equality (with NULL = NULL = true semantics).
    pub fn compare_exactly_equals(&self, other: &Value) -> bool {
        if self.is_null() && other.is_null() {
            return true;
        }
        self.compare_equals(other) == CmpBool::CmpTrue
    }

    // ──────────────────────────────────────────────
    //  Arithmetic Methods
    // ──────────────────────────────────────────────

    pub fn add(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        match self {
            Value::TinyInt(v) => tinyint_modify_op(*v as i64, other, |l, r| l + r),
            Value::SmallInt(v) => smallint_modify_op(*v as i64, other, |l, r| l + r),
            Value::Integer(v) => integer_modify_op(*v as i64, other, |l, r| l + r),
            Value::BigInt(v) => bigint_modify_op(*v, other, |l, r| l + r),
            Value::Decimal(v) => decimal_modify_op(*v, other, |l, r| l + r),
            _ => panic!("Add not implemented for this type"),
        }
    }

    pub fn subtract(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        match self {
            Value::TinyInt(v) => tinyint_modify_op(*v as i64, other, |l, r| l - r),
            Value::SmallInt(v) => smallint_modify_op(*v as i64, other, |l, r| l - r),
            Value::Integer(v) => integer_modify_op(*v as i64, other, |l, r| l - r),
            Value::BigInt(v) => bigint_modify_op(*v, other, |l, r| l - r),
            Value::Decimal(v) => decimal_modify_op(*v, other, |l, r| l - r),
            _ => panic!("Subtract not implemented for this type"),
        }
    }

    pub fn multiply(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        match self {
            Value::TinyInt(v) => tinyint_modify_op(*v as i64, other, |l, r| l * r),
            Value::SmallInt(v) => smallint_modify_op(*v as i64, other, |l, r| l * r),
            Value::Integer(v) => integer_modify_op(*v as i64, other, |l, r| l * r),
            Value::BigInt(v) => bigint_modify_op(*v, other, |l, r| l * r),
            Value::Decimal(v) => decimal_modify_op(*v, other, |l, r| l * r),
            _ => panic!("Multiply not implemented for this type"),
        }
    }

    pub fn divide(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        if other.is_zero() {
            panic!("Division by zero on right-hand side");
        }
        match self {
            Value::TinyInt(v) => tinyint_modify_op(*v as i64, other, |l, r| l / r),
            Value::SmallInt(v) => smallint_modify_op(*v as i64, other, |l, r| l / r),
            Value::Integer(v) => integer_modify_op(*v as i64, other, |l, r| l / r),
            Value::BigInt(v) => bigint_modify_op(*v, other, |l, r| l / r),
            Value::Decimal(v) => decimal_modify_op(*v, other, |l, r| l / r),
            _ => panic!("Divide not implemented for this type"),
        }
    }

    pub fn modulo(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        if other.is_zero() {
            panic!("Division by zero on right-hand side");
        }
        match self {
            Value::TinyInt(v) => tinyint_modulo_op(*v as i8, other),
            Value::SmallInt(v) => smallint_modulo_op(*v as i16, other),
            Value::Integer(v) => integer_modulo_op(*v, other),
            Value::BigInt(v) => bigint_modulo_op(*v, other),
            Value::Decimal(v) => decimal_modulo_op(*v, other),
            _ => panic!("Modulo not implemented for this type"),
        }
    }

    pub fn min_val(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        if self.compare_less_than(other) == CmpBool::CmpTrue {
            self.copy_val()
        } else {
            other.copy_val()
        }
    }

    pub fn max_val(&self, other: &Value) -> Value {
        if self.is_null() || other.is_null() {
            return self.operate_null(other);
        }
        if self.compare_greater_than_equals(other) == CmpBool::CmpTrue {
            self.copy_val()
        } else {
            other.copy_val()
        }
    }

    pub fn sqrt(&self) -> Value {
        if self.is_null() {
            return Value::Decimal(BUSTUB_DECIMAL_NULL);
        }
        match self {
            Value::TinyInt(v) => {
                if *v < 0 {
                    panic!("Cannot take square root of a negative number.");
                }
                Value::Decimal((*v as f64).sqrt())
            }
            Value::SmallInt(v) => {
                if *v < 0 {
                    panic!("Cannot take square root of a negative number.");
                }
                Value::Decimal((*v as f64).sqrt())
            }
            Value::Integer(v) => {
                if *v < 0 {
                    panic!("Cannot take square root of a negative number.");
                }
                Value::Decimal((*v as f64).sqrt())
            }
            Value::BigInt(v) => {
                if *v < 0 {
                    panic!("Cannot take square root of a negative number.");
                }
                Value::Decimal((*v as f64).sqrt())
            }
            Value::Decimal(v) => {
                if *v < 0.0 {
                    panic!("Cannot take square root of a negative number.");
                }
                Value::Decimal(v.sqrt())
            }
            _ => panic!("Sqrt not implemented for this type"),
        }
    }

    pub fn operate_null(&self, _other: &Value) -> Value {
        Value::Nil
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Value::Boolean(v) => *v == 0,
            Value::TinyInt(v) => *v == 0,
            Value::SmallInt(v) => *v == 0,
            Value::Integer(v) => *v == 0,
            Value::BigInt(v) => *v == 0,
            Value::Decimal(v) => *v == 0.0,
            _ => panic!("IsZero not implemented for this type"),
        }
    }

    // ──────────────────────────────────────────────
    //  Serialization
    // ──────────────────────────────────────────────

    pub fn serialize_to(&self, storage: &mut [u8]) {
        match self {
            Value::Nil => {
                // For Nil, we can't determine the type from self alone.
                // The caller should know the expected type and handle this.
                panic!("Cannot serialize a nil value without type context");
            }
            Value::Boolean(v) => {
                storage[..1].copy_from_slice(&v.to_le_bytes());
            }
            Value::TinyInt(v) => {
                storage[..1].copy_from_slice(&v.to_le_bytes());
            }
            Value::SmallInt(v) => {
                storage[..2].copy_from_slice(&v.to_le_bytes());
            }
            Value::Integer(v) => {
                storage[..4].copy_from_slice(&v.to_le_bytes());
            }
            Value::BigInt(v) => {
                storage[..8].copy_from_slice(&v.to_le_bytes());
            }
            Value::Decimal(v) => {
                storage[..8].copy_from_slice(&v.to_le_bytes());
            }
            Value::Timestamp(v) => {
                storage[..8].copy_from_slice(&v.to_le_bytes());
            }
            Value::Varlen(data) => {
                let len = data.len() as u32;
                storage[..4].copy_from_slice(&len.to_le_bytes());
                let copy_len = (len as usize).min(storage.len().saturating_sub(4));
                storage[4..4 + copy_len].copy_from_slice(&data[..copy_len]);
            }
            Value::VarlenRef((ptr, len)) => {
                let len_u32 = *len as u32;
                storage[..4].copy_from_slice(&len_u32.to_le_bytes());
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                let copy_len = (*len).min(storage.len().saturating_sub(4));
                storage[4..4 + copy_len].copy_from_slice(&data[..copy_len]);
            }
        }
    }

    pub fn deserialize_from(storage: &[u8], type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => {
                let val = i8::from_le_bytes(storage[..1].try_into().unwrap());
                if val == BUSTUB_BOOLEAN_NULL {
                    Value::Nil
                } else {
                    Value::Boolean(val)
                }
            }
            TypeId::TinyInt => {
                let val = i8::from_le_bytes(storage[..1].try_into().unwrap());
                if val == BUSTUB_INT8_NULL {
                    Value::Nil
                } else {
                    Value::TinyInt(val)
                }
            }
            TypeId::SmallInt => {
                let val = i16::from_le_bytes(storage[..2].try_into().unwrap());
                if val == BUSTUB_INT16_NULL {
                    Value::Nil
                } else {
                    Value::SmallInt(val)
                }
            }
            TypeId::Integer => {
                let val = i32::from_le_bytes(storage[..4].try_into().unwrap());
                if val == BUSTUB_INT32_NULL {
                    Value::Nil
                } else {
                    Value::Integer(val)
                }
            }
            TypeId::BigInt => {
                let val = i64::from_le_bytes(storage[..8].try_into().unwrap());
                if val == BUSTUB_INT64_NULL {
                    Value::Nil
                } else {
                    Value::BigInt(val)
                }
            }
            TypeId::Decimal => {
                let val = f64::from_le_bytes(storage[..8].try_into().unwrap());
                if val == BUSTUB_DECIMAL_NULL {
                    Value::Nil
                } else {
                    Value::Decimal(val)
                }
            }
            TypeId::Timestamp => {
                let val = u64::from_le_bytes(storage[..8].try_into().unwrap());
                if val == BUSTUB_TIMESTAMP_NULL {
                    Value::Nil
                } else {
                    Value::Timestamp(val)
                }
            }
            TypeId::Varchar => {
                let len = u32::from_le_bytes(storage[..4].try_into().unwrap());
                if len == BUSTUB_VALUE_NULL {
                    return Value::Nil;
                }
                let data = &storage[4..4 + len as usize];
                Value::Varlen(data.to_vec())
            }
        }
    }

    // ──────────────────────────────────────────────
    //  String Conversion / Copy
    // ──────────────────────────────────────────────

    /// Return a string version of this value.
    pub fn to_string_val(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Boolean(v) => match *v {
                1 => "true".to_string(),
                0 => "false".to_string(),
                _ => "boolean_null".to_string(),
            },
            Value::TinyInt(v) => {
                if *v == BUSTUB_INT8_NULL {
                    return "tinyint_null".to_string();
                }
                v.to_string()
            }
            Value::SmallInt(v) => {
                if *v == BUSTUB_INT16_NULL {
                    return "smallint_null".to_string();
                }
                v.to_string()
            }
            Value::Integer(v) => {
                if *v == BUSTUB_INT32_NULL {
                    return "integer_null".to_string();
                }
                v.to_string()
            }
            Value::BigInt(v) => {
                if *v == BUSTUB_INT64_NULL {
                    return "bigint_null".to_string();
                }
                v.to_string()
            }
            Value::Decimal(v) => {
                if *v == BUSTUB_DECIMAL_NULL {
                    return "decimal_null".to_string();
                }
                v.to_string()
            }
            Value::Timestamp(v) => {
                if *v == BUSTUB_TIMESTAMP_NULL {
                    return "timestamp_null".to_string();
                }
                format_timestamp(*v)
            }
            Value::Varlen(data) => {
                let len = data.len();
                if len == BUSTUB_VARCHAR_MAX_LEN as usize {
                    return "varlen_max".to_string();
                }
                if len == 0 {
                    return String::new();
                }
                let text_len = len.saturating_sub(1);
                if text_len > data.len() {
                    return String::new();
                }
                String::from_utf8_lossy(&data[..text_len]).to_string()
            }
            Value::VarlenRef((ptr, len)) => {
                if *len == BUSTUB_VARCHAR_MAX_LEN as usize {
                    return "varlen_max".to_string();
                }
                if *len == 0 {
                    return String::new();
                }
                let text_len = len.saturating_sub(1);
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                if text_len > data.len() {
                    return String::new();
                }
                String::from_utf8_lossy(&data[..text_len]).to_string()
            }
        }
    }

    /// Create a copy of this value.
    pub fn copy_val(&self) -> Value {
        match self {
            Value::Nil => Value::Nil,
            Value::Boolean(v) => Value::Boolean(*v),
            Value::TinyInt(v) => Value::TinyInt(*v),
            Value::SmallInt(v) => Value::SmallInt(*v),
            Value::Integer(v) => Value::Integer(*v),
            Value::BigInt(v) => Value::BigInt(*v),
            Value::Decimal(v) => Value::Decimal(*v),
            Value::Timestamp(v) => Value::Timestamp(*v),
            Value::Varlen(data) => Value::Varlen(data.clone()),
            Value::VarlenRef((ptr, len)) => {
                let data = unsafe { std::slice::from_raw_parts(*ptr, *len) };
                Value::Varlen(data.to_vec())
            }
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_val())
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        self.copy_val()
    }
}

// ─────────────────────────────────────────────────────────────
//  Private helper functions (comparison / arithmetic dispatch)
// ─────────────────────────────────────────────────────────────

/// Compare a boolean value with another value after casting it to boolean.
fn compare_boolean(lhs: i8, rhs: &Value, op: impl FnOnce(i8, i8) -> bool) -> CmpBool {
    let r = rhs.cast_as(TypeId::Boolean);
    match r {
        Value::Boolean(rv) => CmpBool::from(op(lhs, rv)),
        _ => CmpBool::CmpNull,
    }
}

/// Compare a tinyint as i64 with another value.
fn compare_tinyint(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> bool) -> CmpBool {
    match rhs {
        Value::TinyInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::SmallInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Integer(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::BigInt(r) => CmpBool::from(op(lhs, *r)),
        Value::Decimal(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::TinyInt);
            compare_tinyint(lhs, &rv, op)
        }
        _ => CmpBool::CmpNull,
    }
}

/// Compare a smallint as i64 with another value.
fn compare_smallint(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> bool) -> CmpBool {
    match rhs {
        Value::TinyInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::SmallInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Integer(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::BigInt(r) => CmpBool::from(op(lhs, *r)),
        Value::Decimal(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::SmallInt);
            compare_smallint(lhs, &rv, op)
        }
        _ => CmpBool::CmpNull,
    }
}

/// Compare an integer as i64 with another value.
fn compare_integer(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> bool) -> CmpBool {
    match rhs {
        Value::TinyInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::SmallInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Integer(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::BigInt(r) => CmpBool::from(op(lhs, *r)),
        Value::Decimal(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Integer);
            compare_integer(lhs, &rv, op)
        }
        _ => CmpBool::CmpNull,
    }
}

/// Compare a bigint with another value.
fn compare_bigint(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> bool) -> CmpBool {
    match rhs {
        Value::TinyInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::SmallInt(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Integer(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::BigInt(r) => CmpBool::from(op(lhs, *r)),
        Value::Decimal(r) => CmpBool::from(op(lhs, *r as i64)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::BigInt);
            compare_bigint(lhs, &rv, op)
        }
        _ => CmpBool::CmpNull,
    }
}

/// Compare a decimal with another value.
fn compare_decimal(lhs: f64, rhs: &Value, op: impl FnOnce(f64, f64) -> bool) -> CmpBool {
    match rhs {
        Value::TinyInt(r) => CmpBool::from(op(lhs, *r as f64)),
        Value::SmallInt(r) => CmpBool::from(op(lhs, *r as f64)),
        Value::Integer(r) => CmpBool::from(op(lhs, *r as f64)),
        Value::BigInt(r) => CmpBool::from(op(lhs, *r as f64)),
        Value::Decimal(r) => CmpBool::from(op(lhs, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Decimal);
            compare_decimal(lhs, &rv, op)
        }
        _ => CmpBool::CmpNull,
    }
}

/// Compare a timestamp with another value.
fn compare_timestamp(lhs: u64, rhs: &Value, op: impl FnOnce(u64, u64) -> bool) -> CmpBool {
    match rhs {
        Value::Timestamp(r) => CmpBool::from(op(lhs, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Timestamp);
            compare_timestamp(lhs, &rv, op)
        }
        _ => CmpBool::CmpNull,
    }
}

/// Compare a varlen value (given as raw bytes + length) with another value.
fn compare_varlen_with_len(
    data: &[u8],
    len: usize,
    rhs: &Value,
    cmp_op: impl FnOnce(Ordering) -> bool,
) -> CmpBool {
    let str1 = data;
    let len1 = len.saturating_sub(1);

    let (str2, len2) = match rhs {
        Value::Varlen(d) => (d.as_slice(), d.len().saturating_sub(1)),
        Value::VarlenRef((ptr, l)) => {
            let d = unsafe { std::slice::from_raw_parts(*ptr, *l) };
            (d, l.saturating_sub(1))
        }
        _ => {
            let rv = rhs.cast_as(TypeId::Varchar);
            let d = rv.get_data();
            let l = rv.get_storage_size().saturating_sub(1) as usize;
            // Need to extend the lifetime of d to match str1
            // We copy the data to avoid the lifetime issue
            let owned = d.to_vec();
            return compare_varlen_with_len_owned(str1, len1, &owned, l, cmp_op);
        }
    };

    let ordering = compare_strings(str1, len1, str2, len2);
    CmpBool::from(cmp_op(ordering))
}

/// Helper that borrows from an owned Vec to avoid lifetime issues.
fn compare_varlen_with_len_owned(
    str1: &[u8],
    len1: usize,
    owned: &Vec<u8>,
    len2: usize,
    cmp_op: impl FnOnce(Ordering) -> bool,
) -> CmpBool {
    let ordering = compare_strings(str1, len1, owned.as_slice(), len2);
    CmpBool::from(cmp_op(ordering))
}

// ─────────────────────────────────────────────────────────────
//  Private helper functions (arithmetic dispatch)
// ─────────────────────────────────────────────────────────────

/// TinyInt arithmetic: operate as i64 and cast back.
fn tinyint_modify_op(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> i64) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::TinyInt(op(lhs, *r as i64) as i8),
        Value::SmallInt(r) => Value::SmallInt(op(lhs, *r as i64) as i16),
        Value::Integer(r) => Value::Integer(op(lhs, *r as i64) as i32),
        Value::BigInt(r) => Value::BigInt(op(lhs, *r)),
        Value::Decimal(r) => Value::Decimal(op(lhs, *r as i64) as f64),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::TinyInt);
            tinyint_modify_op(lhs, &rv, op)
        }
        _ => panic!("type error"),
    }
}

/// SmallInt arithmetic: operate as i64 and cast back.
fn smallint_modify_op(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> i64) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::SmallInt(op(lhs, *r as i64) as i16),
        Value::SmallInt(r) => Value::SmallInt(op(lhs, *r as i64) as i16),
        Value::Integer(r) => Value::Integer(op(lhs, *r as i64) as i32),
        Value::BigInt(r) => Value::BigInt(op(lhs, *r)),
        Value::Decimal(r) => Value::Decimal(op(lhs, *r as i64) as f64),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::SmallInt);
            smallint_modify_op(lhs, &rv, op)
        }
        _ => panic!("type error"),
    }
}

/// Integer arithmetic: operate as i64 and cast back.
fn integer_modify_op(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> i64) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::Integer(op(lhs, *r as i64) as i32),
        Value::SmallInt(r) => Value::Integer(op(lhs, *r as i64) as i32),
        Value::Integer(r) => Value::Integer(op(lhs, *r as i64) as i32),
        Value::BigInt(r) => Value::BigInt(op(lhs, *r)),
        Value::Decimal(r) => Value::Decimal(op(lhs, *r as i64) as f64),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Integer);
            integer_modify_op(lhs, &rv, op)
        }
        _ => panic!("type error"),
    }
}

/// BigInt arithmetic: operate on i64 directly.
fn bigint_modify_op(lhs: i64, rhs: &Value, op: impl FnOnce(i64, i64) -> i64) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::BigInt(op(lhs, *r as i64)),
        Value::SmallInt(r) => Value::BigInt(op(lhs, *r as i64)),
        Value::Integer(r) => Value::BigInt(op(lhs, *r as i64)),
        Value::BigInt(r) => Value::BigInt(op(lhs, *r)),
        Value::Decimal(r) => Value::Decimal(op(lhs, *r as i64) as f64),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::BigInt);
            bigint_modify_op(lhs, &rv, op)
        }
        _ => panic!("type error"),
    }
}

/// Decimal arithmetic: operate on f64.
fn decimal_modify_op(lhs: f64, rhs: &Value, op: impl FnOnce(f64, f64) -> f64) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::Decimal(op(lhs, *r as f64)),
        Value::SmallInt(r) => Value::Decimal(op(lhs, *r as f64)),
        Value::Integer(r) => Value::Decimal(op(lhs, *r as f64)),
        Value::BigInt(r) => Value::Decimal(op(lhs, *r as f64)),
        Value::Decimal(r) => Value::Decimal(op(lhs, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Decimal);
            decimal_modify_op(lhs, &rv, op)
        }
        _ => panic!("type error"),
    }
}

// ─────────────────────────────────────────────────────────────
//  Modulo operations (have different dispatch per type)
// ─────────────────────────────────────────────────────────────

fn tinyint_modulo_op(lhs: i8, rhs: &Value) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::TinyInt((lhs as i64 % *r as i64) as i8),
        Value::SmallInt(r) => Value::SmallInt((lhs as i64 % *r as i64) as i16),
        Value::Integer(r) => Value::Integer((lhs as i64 % *r as i64) as i32),
        Value::BigInt(r) => Value::BigInt(lhs as i64 % *r),
        Value::Decimal(r) => Value::Decimal(val_mod(lhs as f64, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::TinyInt);
            tinyint_modulo_op(lhs, &rv)
        }
        _ => panic!("type error"),
    }
}

fn smallint_modulo_op(lhs: i16, rhs: &Value) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::SmallInt((lhs as i64 % *r as i64) as i16),
        Value::SmallInt(r) => Value::SmallInt((lhs as i64 % *r as i64) as i16),
        Value::Integer(r) => Value::Integer((lhs as i64 % *r as i64) as i32),
        Value::BigInt(r) => Value::BigInt(lhs as i64 % *r),
        Value::Decimal(r) => Value::Decimal(val_mod(lhs as f64, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::SmallInt);
            smallint_modulo_op(lhs, &rv)
        }
        _ => panic!("type error"),
    }
}

fn integer_modulo_op(lhs: i32, rhs: &Value) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::Integer(lhs % *r as i32),
        Value::SmallInt(r) => Value::Integer(lhs % *r as i32),
        Value::Integer(r) => Value::Integer(lhs % *r),
        Value::BigInt(r) => Value::BigInt(lhs as i64 % *r),
        Value::Decimal(r) => Value::Decimal(val_mod(lhs as f64, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Integer);
            integer_modulo_op(lhs, &rv)
        }
        _ => panic!("type error"),
    }
}

fn bigint_modulo_op(lhs: i64, rhs: &Value) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::BigInt(lhs % *r as i64),
        Value::SmallInt(r) => Value::BigInt(lhs % *r as i64),
        Value::Integer(r) => Value::BigInt(lhs % *r as i64),
        Value::BigInt(r) => Value::BigInt(lhs % *r),
        Value::Decimal(r) => Value::Decimal(val_mod(lhs as f64, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::BigInt);
            bigint_modulo_op(lhs, &rv)
        }
        _ => panic!("type error"),
    }
}

fn decimal_modulo_op(lhs: f64, rhs: &Value) -> Value {
    match rhs {
        Value::TinyInt(r) => Value::Decimal(val_mod(lhs, *r as f64)),
        Value::SmallInt(r) => Value::Decimal(val_mod(lhs, *r as f64)),
        Value::Integer(r) => Value::Decimal(val_mod(lhs, *r as f64)),
        Value::BigInt(r) => Value::Decimal(val_mod(lhs, *r as f64)),
        Value::Decimal(r) => Value::Decimal(val_mod(lhs, *r)),
        Value::Varlen(_) | Value::VarlenRef(_) => {
            let rv = rhs.cast_as(TypeId::Decimal);
            decimal_modulo_op(lhs, &rv)
        }
        _ => panic!("type error"),
    }
}

// ─────────────────────────────────────────────────────────────
//  Other helpers
// ─────────────────────────────────────────────────────────────

fn type_id_to_string(type_id: TypeId) -> String {
    match type_id {
        TypeId::Boolean => "BOOLEAN",
        TypeId::TinyInt => "TINYINT",
        TypeId::SmallInt => "SMALLINT",
        TypeId::Integer => "INTEGER",
        TypeId::BigInt => "BIGINT",
        TypeId::Decimal => "DECIMAL",
        TypeId::Timestamp => "TIMESTAMP",
        TypeId::Varchar => "VARCHAR",
    }
    .to_string()
}

fn format_timestamp(tm: u64) -> String {
    let mut tm = tm;
    let micro = (tm % 1_000_000) as u32;
    tm /= 1_000_000;
    let second = (tm % 100_000) as u32;
    let sec = (second % 60) as u16;
    let second_div = second / 60;
    let min = (second_div % 60) as u16;
    let second_div = second_div / 60;
    let hour = (second_div % 24) as u16;
    tm /= 100_000;
    let year = (tm % 10_000) as u16;
    tm /= 10_000;
    let tz = (tm % 27) as i32 - 12;
    tm /= 27;
    let day = (tm % 32) as u16;
    tm /= 32;
    let month = tm as u16;

    let sign = if tz >= 0 { '+' } else { '-' };
    let tz_abs = tz.abs();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}{}{:02}",
        year, month, day, hour, min, sec, micro, sign, tz_abs
    )
}

