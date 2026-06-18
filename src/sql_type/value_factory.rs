//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// value_factory.rs
//
// Identification: src/sql_type/value_factory.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::sql_type::{get_type_instance, type_id_to_string};
use super::type_id::TypeId;
use super::value::Value;

/// A factory for creating SQL values of various types.
pub struct ValueFactory;

impl ValueFactory {
    /// Clone a value (calls copy).
    pub fn clone_val(src: &Value) -> Value {
        src.copy_val()
    }

    /// Create a TINYINT value.
    pub fn get_tiny_int_value(value: i8) -> Value {
        Value::from_i8(TypeId::TinyInt, value)
    }

    /// Create a SMALLINT value.
    pub fn get_small_int_value(value: i16) -> Value {
        Value::from_i16(TypeId::SmallInt, value)
    }

    /// Create an INTEGER value.
    pub fn get_integer_value(value: i32) -> Value {
        Value::from_i32(TypeId::Integer, value)
    }

    /// Create a BIGINT value.
    pub fn get_big_int_value(value: i64) -> Value {
        Value::from_i64(TypeId::BigInt, value)
    }

    /// Create a TIMESTAMP value.
    pub fn get_timestamp_value(value: i64) -> Value {
        Value::from_i64(TypeId::Timestamp, value)
    }

    /// Create a DECIMAL value.
    pub fn get_decimal_value(value: f64) -> Value {
        Value::from_f64(TypeId::Decimal, value)
    }

    /// Create a VARCHAR value from a string.
    pub fn get_varchar_value(value: &str) -> Value {
        Value::from_str(value)
    }

    /// Create a NULL value of the given type.
    pub fn get_null_value_by_type(type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => Value::from_i8(TypeId::Boolean, BUSTUB_BOOLEAN_NULL),
            TypeId::TinyInt => Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_NULL),
            TypeId::SmallInt => Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_NULL),
            TypeId::Integer => Value::from_i32(TypeId::Integer, BUSTUB_INT32_NULL),
            TypeId::BigInt => Value::from_i64(TypeId::BigInt, BUSTUB_INT64_NULL),
            TypeId::Decimal => Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL),
            TypeId::Varchar => Value::from_bytes(TypeId::Varchar, &[], 0, false),
            _ => panic!("Attempting to create invalid null type"),
        }
    }

    /// Create a zero value of the given type.
    pub fn get_zero_value_by_type(type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => Value::from_i8(TypeId::Boolean, 0),
            TypeId::TinyInt => Value::from_i8(TypeId::TinyInt, 0),
            TypeId::SmallInt => Value::from_i16(TypeId::SmallInt, 0),
            TypeId::Integer => Value::from_i32(TypeId::Integer, 0),
            TypeId::BigInt => Value::from_i64(TypeId::BigInt, 0),
            TypeId::Decimal => Value::from_f64(TypeId::Decimal, 0.0),
            TypeId::Varchar => Value::from_str("0"),
            _ => panic!("Unknown type for get_zero_value_by_type"),
        }
    }

    /// Cast a value to BIGINT.
    pub fn cast_as_big_int(value: &Value) -> Value {
        if get_type_instance(TypeId::BigInt).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_i64(TypeId::BigInt, BUSTUB_INT64_NULL);
            }
            return match value.get_type_id() {
                TypeId::TinyInt => {
                    Value::from_i64(TypeId::BigInt, value.get_as::<i8>() as i64)
                }
                TypeId::SmallInt => {
                    Value::from_i64(TypeId::BigInt, value.get_as::<i16>() as i64)
                }
                TypeId::Integer => {
                    Value::from_i64(TypeId::BigInt, value.get_as::<i32>() as i64)
                }
                TypeId::BigInt => {
                    Value::from_i64(TypeId::BigInt, value.get_as::<i64>())
                }
                TypeId::Decimal => {
                    let v = value.get_as::<f64>();
                    if v > BUSTUB_INT64_MAX as f64 || v < BUSTUB_INT64_MIN as f64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i64(TypeId::BigInt, v as i64)
                }
                TypeId::Varchar => {
                    let s = value.to_string_val();
                    let bigint: i64 = s
                        .parse()
                        .unwrap_or_else(|_| panic!("Invalid input syntax for bigint: '{}'", s));
                    if bigint > BUSTUB_INT64_MAX || bigint < BUSTUB_INT64_MIN {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i64(TypeId::BigInt, bigint)
                }
                _ => panic!(
                    "{} is not coercable to BIGINT.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to BIGINT.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to INTEGER.
    pub fn cast_as_integer(value: &Value) -> Value {
        if get_type_instance(TypeId::Integer).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_i32(TypeId::Integer, BUSTUB_INT32_NULL);
            }
            return match value.get_type_id() {
                TypeId::TinyInt => {
                    Value::from_i32(TypeId::Integer, value.get_as::<i8>() as i32)
                }
                TypeId::SmallInt => {
                    Value::from_i32(TypeId::Integer, value.get_as::<i16>() as i32)
                }
                TypeId::Integer => {
                    Value::from_i32(TypeId::Integer, value.get_as::<i32>())
                }
                TypeId::BigInt => {
                    let v = value.get_as::<i64>();
                    if v > BUSTUB_INT32_MAX as i64 || v < BUSTUB_INT32_MIN as i64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i32(TypeId::Integer, v as i32)
                }
                TypeId::Decimal => {
                    let v = value.get_as::<f64>();
                    if v > BUSTUB_INT32_MAX as f64 || v < BUSTUB_INT32_MIN as f64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i32(TypeId::Integer, v as i32)
                }
                TypeId::Varchar => {
                    let s = value.to_string_val();
                    let integer: i32 = s
                        .parse()
                        .unwrap_or_else(|_| panic!("Invalid input syntax for integer: '{}'", s));
                    if integer > BUSTUB_INT32_MAX || integer < BUSTUB_INT32_MIN {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i32(TypeId::Integer, integer)
                }
                _ => panic!(
                    "{} is not coercable to INTEGER.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to INTEGER.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to SMALLINT.
    pub fn cast_as_small_int(value: &Value) -> Value {
        if get_type_instance(TypeId::SmallInt).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_NULL);
            }
            return match value.get_type_id() {
                TypeId::TinyInt => {
                    Value::from_i16(TypeId::SmallInt, value.get_as::<i8>() as i16)
                }
                TypeId::SmallInt => {
                    Value::from_i16(TypeId::SmallInt, value.get_as::<i16>())
                }
                TypeId::Integer => {
                    let v = value.get_as::<i32>();
                    if v > BUSTUB_INT16_MAX as i32 || v < BUSTUB_INT16_MIN as i32 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i16(TypeId::SmallInt, v as i16)
                }
                TypeId::BigInt => {
                    let v = value.get_as::<i64>();
                    if v > BUSTUB_INT16_MAX as i64 || v < BUSTUB_INT16_MIN as i64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i16(TypeId::SmallInt, v as i16)
                }
                TypeId::Decimal => {
                    let v = value.get_as::<f64>();
                    if v > BUSTUB_INT16_MAX as f64 || v < BUSTUB_INT16_MIN as f64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i16(TypeId::SmallInt, v as i16)
                }
                TypeId::Varchar => {
                    let s = value.to_string_val();
                    let smallint: i16 = s.parse().unwrap_or_else(|_| {
                        panic!("Invalid input syntax for smallint: '{}'", s)
                    });
                    if smallint < BUSTUB_INT16_MIN {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i16(TypeId::SmallInt, smallint)
                }
                _ => panic!(
                    "{} is not coercable to SMALLINT.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to SMALLINT.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to TINYINT.
    pub fn cast_as_tiny_int(value: &Value) -> Value {
        if get_type_instance(TypeId::TinyInt).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_NULL);
            }
            return match value.get_type_id() {
                TypeId::TinyInt => Value::from_i8(TypeId::TinyInt, value.get_as::<i8>()),
                TypeId::SmallInt => {
                    let v = value.get_as::<i16>();
                    if v > BUSTUB_INT8_MAX as i16 || v < BUSTUB_INT8_MIN as i16 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i8(TypeId::TinyInt, v as i8)
                }
                TypeId::Integer => {
                    let v = value.get_as::<i32>();
                    if v > BUSTUB_INT8_MAX as i32 || v < BUSTUB_INT8_MIN as i32 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i8(TypeId::TinyInt, v as i8)
                }
                TypeId::BigInt => {
                    let v = value.get_as::<i64>();
                    if v > BUSTUB_INT8_MAX as i64 || v < BUSTUB_INT8_MIN as i64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i8(TypeId::TinyInt, v as i8)
                }
                TypeId::Decimal => {
                    let v = value.get_as::<f64>();
                    if v > BUSTUB_INT8_MAX as f64 || v < BUSTUB_INT8_MIN as f64 {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i8(TypeId::TinyInt, v as i8)
                }
                TypeId::Varchar => {
                    let s = value.to_string_val();
                    let tinyint: i8 = s.parse().unwrap_or_else(|_| {
                        panic!("Invalid input syntax for tinyint: '{}'", s)
                    });
                    if tinyint < BUSTUB_INT8_MIN {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_i8(TypeId::TinyInt, tinyint)
                }
                _ => panic!(
                    "{} is not coercable to TINYINT.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to TINYINT.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to DECIMAL.
    pub fn cast_as_decimal(value: &Value) -> Value {
        if get_type_instance(TypeId::Decimal).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL);
            }
            return match value.get_type_id() {
                TypeId::TinyInt => {
                    Value::from_f64(TypeId::Decimal, value.get_as::<i8>() as f64)
                }
                TypeId::SmallInt => {
                    Value::from_f64(TypeId::Decimal, value.get_as::<i16>() as f64)
                }
                TypeId::Integer => {
                    Value::from_f64(TypeId::Decimal, value.get_as::<i32>() as f64)
                }
                TypeId::BigInt => {
                    Value::from_f64(TypeId::Decimal, value.get_as::<i64>() as f64)
                }
                TypeId::Decimal => Value::from_f64(TypeId::Decimal, value.get_as::<f64>()),
                TypeId::Varchar => {
                    let s = value.to_string_val();
                    let res: f64 = s.parse().unwrap_or_else(|_| {
                        panic!("Invalid input syntax for decimal: '{}'", s)
                    });
                    if res > BUSTUB_DECIMAL_MAX || res < BUSTUB_DECIMAL_MIN {
                        panic!("Numeric value out of range.");
                    }
                    Value::from_f64(TypeId::Decimal, res)
                }
                _ => panic!(
                    "{} is not coercable to DECIMAL.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to DECIMAL.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to VARCHAR.
    pub fn cast_as_varchar(value: &Value) -> Value {
        if get_type_instance(TypeId::Varchar).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_bytes(TypeId::Varchar, &[], 0, false);
            }
            return Value::from_str(&value.to_string_val());
        }
        panic!(
            "{} is not coercable to VARCHAR.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to TIMESTAMP.
    pub fn cast_as_timestamp(value: &Value) -> Value {
        if get_type_instance(TypeId::Timestamp).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_i64(TypeId::Timestamp, BUSTUB_TIMESTAMP_NULL as i64);
            }
            return match value.get_type_id() {
                TypeId::Timestamp => {
                    Value::from_u64(TypeId::Timestamp, value.get_as::<u64>())
                }
                TypeId::Varchar => {
                    let str_val = value.to_string_val();
                    let s = if str_val.len() == 22 {
                        format!(
                            "{}.000000{}",
                            &str_val[..19],
                            &str_val[19..22]
                        )
                    } else {
                        str_val
                    };

                    if s.len() != 29 {
                        panic!("Timestamp format error.");
                    }

                    // Validate timestamp format
                    let chars: Vec<char> = s.chars().collect();
                    let is_digit: [bool; 29] = [
                        true, true, true, true, false, true, true, false, true, true, false,
                        true, true, false, true, true, false, true, true, false, true, true,
                        true, true, true, true, false, true, true,
                    ];

                    for i in 0..29 {
                        if is_digit[i] && !chars[i].is_ascii_digit() {
                            panic!("Timestamp format error.");
                        }
                    }

                    if chars[10] != ' '
                        || chars[4] != '-'
                        || chars[7] != '-'
                        || chars[13] != ':'
                        || chars[16] != ':'
                        || chars[19] != '.'
                        || (chars[26] != '+' && chars[26] != '-')
                    {
                        panic!("Timestamp format error.");
                    }

                    let year: u32 = s[..4].parse().unwrap();
                    let month: u32 = s[5..7].parse().unwrap();
                    let day: u32 = s[8..10].parse().unwrap();
                    let hour: u32 = s[11..13].parse().unwrap();
                    let min: u32 = s[14..16].parse().unwrap();
                    let sec: u32 = s[17..19].parse().unwrap();
                    let micro: u32 = s[20..26].parse().unwrap();
                    let tz: i32 = s[26..29].parse().unwrap();

                    if year > 9999
                        || month > 12
                        || day > 31
                        || hour > 23
                        || min > 59
                        || sec > 59
                        || micro > 999999
                        || day == 0
                        || month == 0
                    {
                        panic!("Timestamp value out of range.");
                    }

                    let max_day = [0u32, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                    let max_day_lunar =
                        [0u32, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

                    let is_leap =
                        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
                    if (is_leap && day > max_day_lunar[month as usize])
                        || (!is_leap && day > max_day[month as usize])
                    {
                        panic!("Timestamp value out of range.");
                    }

                    let timezone = tz + 12;
                    if tz > 26 {
                        panic!("Timestamp format error.");
                    }

                    let mut res: u64 = 0;
                    res += month as u64;
                    res *= 32;
                    res += day as u64;
                    res *= 27;
                    res += timezone as u64;
                    res *= 10_000;
                    res += year as u64;
                    res *= 100_000;
                    res += (hour * 3600 + min * 60 + sec) as u64;
                    res *= 1_000_000;
                    res += micro as u64;
                    Value::from_u64(TypeId::Timestamp, res)
                }
                _ => panic!(
                    "{} is not coercable to TIMESTAMP.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to TIMESTAMP.",
            type_id_to_string(value.get_type_id())
        )
    }

    /// Cast a value to BOOLEAN.
    pub fn cast_as_boolean(value: &Value) -> Value {
        if get_type_instance(TypeId::Boolean).is_coercable_from(value.get_type_id()) {
            if value.is_null() {
                return Value::from_i8(TypeId::Boolean, BUSTUB_BOOLEAN_NULL);
            }
            return match value.get_type_id() {
                TypeId::Boolean => Value::from_i8(TypeId::Boolean, value.get_as::<i8>()),
                TypeId::Varchar => {
                    let s = value.to_string_val().to_lowercase();
                    if s == "true" || s == "1" || s == "t" {
                        return Value::from_i8(TypeId::Boolean, 1);
                    }
                    if s == "false" || s == "0" || s == "f" {
                        return Value::from_i8(TypeId::Boolean, 0);
                    }
                    panic!("Boolean value format error.");
                }
                _ => panic!(
                    "{} is not coercable to BOOLEAN.",
                    type_id_to_string(value.get_type_id())
                ),
            };
        }
        panic!(
            "{} is not coercable to BOOLEAN.",
            type_id_to_string(value.get_type_id())
        )
    }
}


