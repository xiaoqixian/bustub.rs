//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// decimal_type.rs
//
// Identification: src/sql_type/decimal_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::numeric_type;
use super::sql_type::{get_cmp_bool, CmpBool, SqlType, type_id_to_string};
use super::type_id::TypeId;
use super::value::Value;

/// The SQL DECIMAL type (64-bit floating point).
pub struct DecimalType {
    type_id: TypeId,
}

impl DecimalType {
    pub fn new() -> Self {
        DecimalType {
            type_id: TypeId::Decimal,
        }
    }

    fn compare_op(left: &Value, right: &Value, op: impl FnOnce(f64, f64) -> bool) -> CmpBool {
        match right.get_type_id() {
            TypeId::TinyInt => get_cmp_bool(op(left.get_as::<f64>(), right.get_as::<i8>() as f64)),
            TypeId::SmallInt => get_cmp_bool(op(left.get_as::<f64>(), right.get_as::<i16>() as f64)),
            TypeId::Integer => get_cmp_bool(op(left.get_as::<f64>(), right.get_as::<i32>() as f64)),
            TypeId::BigInt => get_cmp_bool(op(left.get_as::<f64>(), right.get_as::<i64>() as f64)),
            TypeId::Decimal => get_cmp_bool(op(left.get_as::<f64>(), right.get_as::<f64>())),
            TypeId::Varchar => {
                let r_value = right.cast_as(TypeId::Decimal);
                get_cmp_bool(op(left.get_as::<f64>(), r_value.get_as::<f64>()))
            }
            _ => CmpBool::CmpNull,
        }
    }

    fn modify_op(left: &Value, right: &Value, op: impl FnOnce(f64, f64) -> f64) -> Value {
        match right.get_type_id() {
            TypeId::TinyInt => Value::from_f64(TypeId::Decimal, op(left.get_as::<f64>(), right.get_as::<i8>() as f64)),
            TypeId::SmallInt => Value::from_f64(TypeId::Decimal, op(left.get_as::<f64>(), right.get_as::<i16>() as f64)),
            TypeId::Integer => Value::from_f64(TypeId::Decimal, op(left.get_as::<f64>(), right.get_as::<i32>() as f64)),
            TypeId::BigInt => Value::from_f64(TypeId::Decimal, op(left.get_as::<f64>(), right.get_as::<i64>() as f64)),
            TypeId::Decimal => Value::from_f64(TypeId::Decimal, op(left.get_as::<f64>(), right.get_as::<f64>())),
            TypeId::Varchar => {
                let r_value = right.cast_as(TypeId::Decimal);
                Value::from_f64(TypeId::Decimal, op(left.get_as::<f64>(), r_value.get_as::<f64>()))
            }
            _ => panic!("type error"),
        }
    }
}

impl SqlType for DecimalType {
    fn get_type_id(&self) -> TypeId { self.type_id }
    fn get_type_size(&self) -> u64 { 8 }
    fn is_coercable_from(&self, type_id: TypeId) -> bool {
        matches!(type_id, TypeId::TinyInt | TypeId::SmallInt | TypeId::Integer | TypeId::BigInt | TypeId::Decimal | TypeId::Varchar)
    }
    fn to_string_id(&self) -> String { type_id_to_string(self.type_id) }
    fn get_min_value(&self) -> Value { Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_MIN) }
    fn get_max_value(&self) -> Value { Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_MAX) }

    fn add(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        Self::modify_op(left, right, |l, r| l + r)
    }

    fn subtract(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        Self::modify_op(left, right, |l, r| l - r)
    }

    fn multiply(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        Self::modify_op(left, right, |l, r| l * r)
    }

    fn divide(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        if right.is_zero() { panic!("Division by zero on right-hand side"); }
        Self::modify_op(left, right, |l, r| l / r)
    }

    fn modulo(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return self.operate_null(left, right); }
        if right.is_zero() { panic!("Division by zero on right-hand side"); }
        match right.get_type_id() {
            TypeId::TinyInt => Value::from_f64(TypeId::Decimal, numeric_type::val_mod(left.get_as::<f64>(), right.get_as::<i8>() as f64)),
            TypeId::SmallInt => Value::from_f64(TypeId::Decimal, numeric_type::val_mod(left.get_as::<f64>(), right.get_as::<i16>() as f64)),
            TypeId::Integer => Value::from_f64(TypeId::Decimal, numeric_type::val_mod(left.get_as::<f64>(), right.get_as::<i32>() as f64)),
            TypeId::BigInt => Value::from_f64(TypeId::Decimal, numeric_type::val_mod(left.get_as::<f64>(), right.get_as::<i64>() as f64)),
            TypeId::Decimal => Value::from_f64(TypeId::Decimal, numeric_type::val_mod(left.get_as::<f64>(), right.get_as::<f64>())),
            TypeId::Varchar => {
                let r_value = right.cast_as(TypeId::Decimal);
                Value::from_f64(TypeId::Decimal, numeric_type::val_mod(left.get_as::<f64>(), r_value.get_as::<f64>()))
            }
            _ => panic!("type error"),
        }
    }

    fn min_val(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        if left.compare_less_than_equals(right) == CmpBool::CmpTrue { self.copy(left) } else { self.copy(right) }
    }

    fn max_val(&self, left: &Value, right: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return left.operate_null(right); }
        if left.compare_greater_than_equals(right) == CmpBool::CmpTrue { self.copy(left) } else { self.copy(right) }
    }

    fn sqrt(&self, val: &Value) -> Value {
        assert!(self.get_type_id() == TypeId::Decimal);
        if val.is_null() { return Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL); }
        let v = val.get_as::<f64>();
        if v < 0.0 { panic!("Cannot take square root of a negative number."); }
        Value::from_f64(TypeId::Decimal, v.sqrt())
    }

    fn operate_null(&self, _left: &Value, _right: &Value) -> Value {
        Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL)
    }

    fn is_zero(&self, val: &Value) -> bool {
        assert!(self.get_type_id() == TypeId::Decimal);
        val.get_as::<f64>() == 0.0
    }

    fn is_inlined(&self, _val: &Value) -> bool { true }

    fn to_string_val(&self, val: &Value) -> String {
        if val.is_null() { return "decimal_null".to_string(); }
        val.get_as::<f64>().to_string()
    }

    fn serialize_to(&self, val: &Value, storage: &mut [u8]) {
        let v: f64 = val.get_as();
        storage[..8].copy_from_slice(&v.to_le_bytes());
    }

    fn deserialize_from(&self, storage: &[u8]) -> Value {
        let val = f64::from_le_bytes(storage[..8].try_into().unwrap());
        Value::from_f64(TypeId::Decimal, val)
    }

    fn copy(&self, val: &Value) -> Value {
        Value::from_f64(TypeId::Decimal, val.get_as::<f64>())
    }

    fn cast_as(&self, val: &Value, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => {
                if val.is_null() { return Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_NULL); }
                let v = val.get_as::<f64>();
                if v > BUSTUB_INT8_MAX as f64 || v < BUSTUB_INT8_MIN as f64 { panic!("Numeric value out of range."); }
                Value::from_i8(TypeId::TinyInt, v as i8)
            }
            TypeId::SmallInt => {
                if val.is_null() { return Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_NULL); }
                let v = val.get_as::<f64>();
                if v > BUSTUB_INT16_MAX as f64 || v < BUSTUB_INT16_MIN as f64 { panic!("Numeric value out of range."); }
                Value::from_i16(TypeId::SmallInt, v as i16)
            }
            TypeId::Integer => {
                if val.is_null() { return Value::from_i32(TypeId::Integer, BUSTUB_INT32_NULL); }
                let v = val.get_as::<f64>();
                if v > BUSTUB_INT32_MAX as f64 || v < BUSTUB_INT32_MIN as f64 { panic!("Numeric value out of range."); }
                Value::from_i32(TypeId::Integer, v as i32)
            }
            TypeId::BigInt => {
                if val.is_null() { return Value::from_i64(TypeId::BigInt, BUSTUB_INT64_NULL); }
                let v = val.get_as::<f64>();
                if v >= BUSTUB_INT64_MAX as f64 || v < BUSTUB_INT64_MIN as f64 { panic!("Numeric value out of range."); }
                Value::from_i64(TypeId::BigInt, v as i64)
            }
            TypeId::Decimal => val.copy_val(),
            TypeId::Varchar => {
                if val.is_null() { return Value::from_bytes(TypeId::Varchar, &[], 0, false); }
                Value::from_str(&val.to_string_val())
            }
            _ => panic!("DECIMAL is not coercable to {}", type_id_to_string(type_id)),
        }
    }

    fn compare_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Decimal); assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        Self::compare_op(left, right, |l, r| (l - r).abs() < f64::EPSILON)
    }
    fn compare_not_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Decimal); assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        Self::compare_op(left, right, |l, r| (l - r).abs() >= f64::EPSILON)
    }
    fn compare_less_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Decimal); assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        Self::compare_op(left, right, |l, r| l < r)
    }
    fn compare_less_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Decimal); assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        Self::compare_op(left, right, |l, r| l <= r)
    }
    fn compare_greater_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Decimal); assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        Self::compare_op(left, right, |l, r| l > r)
    }
    fn compare_greater_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Decimal); assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() { return CmpBool::CmpNull; }
        Self::compare_op(left, right, |l, r| l >= r)
    }

    fn get_data<'a>(&self, _val: &'a Value) -> &'a [u8] { panic!("GetData not implemented for DecimalType") }
    fn get_storage_size(&self, _val: &Value) -> u32 { panic!("GetStorageSize not implemented for DecimalType") }
}


