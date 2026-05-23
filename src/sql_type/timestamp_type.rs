//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// timestamp_type.rs
//
// Identification: src/sql_type/timestamp_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::sql_type::{get_cmp_bool, CmpBool, SqlType, type_id_to_string};
use super::type_id::TypeId;
use super::value::Value;

/// The SQL TIMESTAMP type (64-bit unsigned integer).
pub struct TimestampType {
    type_id: TypeId,
}

impl TimestampType {
    pub const K_USECS_PER_DATE: u64 = 86_400_000_000;

    pub fn new() -> Self {
        TimestampType {
            type_id: TypeId::Timestamp,
        }
    }
}

impl SqlType for TimestampType {
    fn get_type_id(&self) -> TypeId { self.type_id }
    fn get_type_size(&self) -> u64 { 8 }
    fn is_coercable_from(&self, type_id: TypeId) -> bool {
        type_id == TypeId::Varchar || type_id == TypeId::Timestamp
    }
    fn to_string_id(&self) -> String { type_id_to_string(self.type_id) }
    fn get_min_value(&self) -> Value { Value::from_u64(TypeId::Timestamp, 0) }
    fn get_max_value(&self) -> Value { Value::from_u64(TypeId::Timestamp, BUSTUB_TIMESTAMP_MAX) }

    fn compare_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        get_cmp_bool(left.get_as::<u64>() == right.get_as::<u64>())
    }

    fn compare_not_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        get_cmp_bool(left.get_as::<u64>() != right.get_as::<u64>())
    }

    fn compare_less_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        get_cmp_bool(left.get_as::<u64>() < right.get_as::<u64>())
    }

    fn compare_less_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        get_cmp_bool(left.get_as::<u64>() <= right.get_as::<u64>())
    }

    fn compare_greater_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        get_cmp_bool((left.get_as::<u64>() as i64) > (right.get_as::<u64>() as i64))
    }

    fn compare_greater_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        get_cmp_bool(left.get_as::<u64>() >= right.get_as::<u64>())
    }

    fn add(&self, _left: &Value, _right: &Value) -> Value { panic!("Add not implemented for TimestampType") }
    fn subtract(&self, _left: &Value, _right: &Value) -> Value { panic!("Subtract not implemented for TimestampType") }
    fn multiply(&self, _left: &Value, _right: &Value) -> Value { panic!("Multiply not implemented for TimestampType") }
    fn divide(&self, _left: &Value, _right: &Value) -> Value { panic!("Divide not implemented for TimestampType") }
    fn modulo(&self, _left: &Value, _right: &Value) -> Value { panic!("Modulo not implemented for TimestampType") }

    fn min_val(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        if left.compare_less_than(right) == CmpBool::CmpTrue { self.copy(left) } else { self.copy(right) }
    }

    fn max_val(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        if left.compare_greater_than_equals(right) == CmpBool::CmpTrue { self.copy(left) } else { self.copy(right) }
    }

    fn sqrt(&self, _val: &Value) -> Value { panic!("Sqrt not implemented for TimestampType") }

    fn operate_null(&self, _left: &Value, _right: &Value) -> Value {
        Value::from_u64(TypeId::Timestamp, BUSTUB_TIMESTAMP_NULL)
    }

    fn is_zero(&self, _val: &Value) -> bool { panic!("IsZero not implemented for TimestampType") }
    fn is_inlined(&self, _val: &Value) -> bool { true }

    fn to_string_val(&self, val: &Value) -> String {
        if val.is_null() { return "timestamp_null".to_string(); }
        let mut tm = val.get_as::<u64>();
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

    fn serialize_to(&self, val: &Value, storage: &mut [u8]) {
        let v: u64 = val.get_as();
        storage[..8].copy_from_slice(&v.to_le_bytes());
    }

    fn deserialize_from(&self, storage: &[u8]) -> Value {
        let val = u64::from_le_bytes(storage[..8].try_into().unwrap());
        Value::from_u64(TypeId::Timestamp, val)
    }

    fn copy(&self, val: &Value) -> Value {
        Value::from_u64(TypeId::Timestamp, val.get_as::<u64>())
    }

    fn cast_as(&self, val: &Value, type_id: TypeId) -> Value {
        match type_id {
            TypeId::Timestamp => self.copy(val),
            TypeId::Varchar => {
                if val.is_null() {
                    return Value::from_bytes(TypeId::Varchar, &[], 0, false);
                }
                Value::from_string(TypeId::Varchar, &val.to_string_val())
            }
            _ => panic!(
                "TIMESTAMP is not coercable to {}",
                type_id_to_string(type_id)
            ),
        }
    }

    fn get_data<'a>(&self, _val: &'a Value) -> &'a [u8] { panic!("GetData not implemented for TimestampType") }
    fn get_storage_size(&self, _val: &Value) -> u32 { panic!("GetStorageSize not implemented for TimestampType") }
}


