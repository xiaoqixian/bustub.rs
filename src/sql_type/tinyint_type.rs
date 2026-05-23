//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// tinyint_type.rs
//
// Identification: src/sql_type/tinyint_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::numeric_type::NumericTypeImpl;
use super::sql_type::{get_cmp_bool, CmpBool, SqlType, type_id_to_string};
use super::type_id::TypeId;
use super::value::Value;

/// The SQL TINYINT type (1-byte signed integer).
pub struct TinyintType {
    inner: NumericTypeImpl,
}

impl TinyintType {
    pub fn new() -> Self {
        TinyintType {
            inner: NumericTypeImpl::new(TypeId::TinyInt),
        }
    }

    /// Macro-like helper for comparing tinyint with other types.
    fn compare_op(left: &Value, right: &Value, op: impl FnOnce(i64, i64) -> bool) -> CmpBool {
        match right.get_type_id() {
            TypeId::TinyInt => get_cmp_bool(op(left.get_as::<i8>() as i64, right.get_as::<i8>() as i64)),
            TypeId::SmallInt => get_cmp_bool(op(left.get_as::<i8>() as i64, right.get_as::<i16>() as i64)),
            TypeId::Integer => get_cmp_bool(op(left.get_as::<i8>() as i64, right.get_as::<i32>() as i64)),
            TypeId::BigInt => get_cmp_bool(op(left.get_as::<i8>() as i64, right.get_as::<i64>())),
            TypeId::Decimal => get_cmp_bool(op(left.get_as::<i8>() as f64 as i64, right.get_as::<f64>() as i64)),
            TypeId::Varchar => {
                let r_value = right.cast_as(TypeId::TinyInt);
                get_cmp_bool(op(left.get_as::<i8>() as i64, r_value.get_as::<i8>() as i64))
            }
            _ => CmpBool::CmpNull,
        }
    }

    /// Helper for arithmetic operations on tinyint.
    fn modify_op(left: &Value, right: &Value, op: impl FnOnce(i64, i64) -> i64) -> Value {
        match right.get_type_id() {
            TypeId::TinyInt => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i8>() as i64;
                Value::from_i8(TypeId::TinyInt, op(l, r) as i8)
            }
            TypeId::SmallInt => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i16>() as i64;
                Value::from_i16(TypeId::SmallInt, op(l, r) as i16)
            }
            TypeId::Integer => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i32>() as i64;
                Value::from_i32(TypeId::Integer, op(l, r) as i32)
            }
            TypeId::BigInt => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i64>();
                Value::from_i64(TypeId::BigInt, op(l, r))
            }
            TypeId::Decimal => {
                let l = left.get_as::<i8>() as f64;
                let r = right.get_as::<f64>();
                Value::from_f64(TypeId::Decimal, op(l as i64, r as i64) as f64)
            }
            TypeId::Varchar => {
                let r_value = right.cast_as(TypeId::TinyInt);
                let l = left.get_as::<i8>() as i64;
                let r = r_value.get_as::<i8>() as i64;
                Value::from_i8(TypeId::TinyInt, op(l, r) as i8)
            }
            _ => panic!("type error"),
        }
    }
}

impl SqlType for TinyintType {
    fn get_type_id(&self) -> TypeId {
        self.inner.type_id
    }

    fn get_type_size(&self) -> u64 {
        1
    }

    fn is_coercable_from(&self, type_id: TypeId) -> bool {
        matches!(
            type_id,
            TypeId::TinyInt
                | TypeId::SmallInt
                | TypeId::Integer
                | TypeId::BigInt
                | TypeId::Decimal
                | TypeId::Varchar
        )
    }

    fn to_string_id(&self) -> String {
        type_id_to_string(self.inner.type_id)
    }

    fn get_min_value(&self) -> Value {
        Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_MIN)
    }

    fn get_max_value(&self) -> Value {
        Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_MAX)
    }

    fn add(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }
        Self::modify_op(left, right, |l, r| l + r)
    }

    fn subtract(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }
        Self::modify_op(left, right, |l, r| l - r)
    }

    fn multiply(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }
        Self::modify_op(left, right, |l, r| l * r)
    }

    fn divide(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }

        if right.is_zero() {
            panic!("Division by zero on right-hand side");
        }

        Self::modify_op(left, right, |l, r| l / r)
    }

    fn modulo(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }

        if right.is_zero() {
            panic!("Division by zero on right-hand side");
        }

        match right.get_type_id() {
            TypeId::TinyInt => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i8>() as i64;
                Value::from_i8(TypeId::TinyInt, (l % r) as i8)
            }
            TypeId::SmallInt => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i16>() as i64;
                Value::from_i16(TypeId::SmallInt, (l % r) as i16)
            }
            TypeId::Integer => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i32>() as i64;
                Value::from_i32(TypeId::Integer, (l % r) as i32)
            }
            TypeId::BigInt => {
                let l = left.get_as::<i8>() as i64;
                let r = right.get_as::<i64>();
                Value::from_i64(TypeId::BigInt, l % r)
            }
            TypeId::Decimal => {
                let l = left.get_as::<i8>() as f64;
                let r = right.get_as::<f64>();
                Value::from_f64(TypeId::Decimal, l % r)
            }
            TypeId::Varchar => {
                let r_value = right.cast_as(TypeId::TinyInt);
                let l = left.get_as::<i8>() as i64;
                let r = r_value.get_as::<i8>() as i64;
                Value::from_i8(TypeId::TinyInt, (l % r) as i8)
            }
            _ => panic!("type error"),
        }
    }

    fn sqrt(&self, val: &Value) -> Value {
        assert!(val.check_integer());
        if val.is_null() {
            return Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL);
        }

        let v = val.get_as::<i8>();
        if v < 0 {
            panic!("Cannot take square root of a negative number.");
        }
        Value::from_f64(TypeId::Decimal, (v as f64).sqrt())
    }

    fn min_val(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }
        if left.compare_less_than(right) == CmpBool::CmpTrue {
            self.copy(left)
        } else {
            self.copy(right)
        }
    }

    fn max_val(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return self.operate_null(left, right);
        }
        if left.compare_greater_than_equals(right) == CmpBool::CmpTrue {
            self.copy(left)
        } else {
            self.copy(right)
        }
    }

    fn operate_null(&self, _left: &Value, right: &Value) -> Value {
        match right.get_type_id() {
            TypeId::TinyInt => Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_NULL),
            TypeId::SmallInt => Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_NULL),
            TypeId::Integer => Value::from_i32(TypeId::Integer, BUSTUB_INT32_NULL),
            TypeId::BigInt => Value::from_i64(TypeId::BigInt, BUSTUB_INT64_NULL),
            TypeId::Decimal => Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL),
            _ => panic!("type error"),
        }
    }

    fn is_zero(&self, val: &Value) -> bool {
        val.get_as::<i8>() == 0
    }

    fn is_inlined(&self, _val: &Value) -> bool {
        true
    }

    fn to_string_val(&self, val: &Value) -> String {
        assert!(val.check_integer());
        if val.is_null() {
            return "tinyint_null".to_string();
        }
        val.get_as::<i8>().to_string()
    }

    fn serialize_to(&self, val: &Value, storage: &mut [u8]) {
        let v: i8 = val.get_as();
        storage[..1].copy_from_slice(&v.to_le_bytes());
    }

    fn deserialize_from(&self, storage: &[u8]) -> Value {
        let val = i8::from_le_bytes(storage[..1].try_into().unwrap());
        Value::from_i8(TypeId::TinyInt, val)
    }

    fn copy(&self, val: &Value) -> Value {
        assert!(val.check_integer());
        Value::from_i8(TypeId::TinyInt, val.get_as::<i8>())
    }

    fn cast_as(&self, val: &Value, type_id: TypeId) -> Value {
        match type_id {
            TypeId::TinyInt => {
                if val.is_null() {
                    return Value::from_i8(TypeId::TinyInt, BUSTUB_INT8_NULL);
                }
                self.copy(val)
            }
            TypeId::SmallInt => {
                if val.is_null() {
                    return Value::from_i16(TypeId::SmallInt, BUSTUB_INT16_NULL);
                }
                Value::from_i16(TypeId::SmallInt, val.get_as::<i8>() as i16)
            }
            TypeId::Integer => {
                if val.is_null() {
                    return Value::from_i32(TypeId::Integer, BUSTUB_INT32_NULL);
                }
                Value::from_i32(TypeId::Integer, val.get_as::<i8>() as i32)
            }
            TypeId::BigInt => {
                if val.is_null() {
                    return Value::from_i64(TypeId::BigInt, BUSTUB_INT64_NULL);
                }
                Value::from_i64(TypeId::BigInt, val.get_as::<i8>() as i64)
            }
            TypeId::Decimal => {
                if val.is_null() {
                    return Value::from_f64(TypeId::Decimal, BUSTUB_DECIMAL_NULL);
                }
                Value::from_f64(TypeId::Decimal, val.get_as::<i8>() as f64)
            }
            TypeId::Varchar => {
                if val.is_null() {
                    return Value::from_bytes(TypeId::Varchar, &[], 0, false);
                }
                Value::from_string(TypeId::Varchar, &val.to_string_val())
            }
            _ => {
                panic!(
                    "tinyint is not coercable to {}",
                    type_id_to_string(type_id)
                )
            }
        }
    }

    // Comparison functions
    fn compare_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        Self::compare_op(left, right, |l, r| l == r)
    }

    fn compare_not_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        Self::compare_op(left, right, |l, r| l != r)
    }

    fn compare_less_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        Self::compare_op(left, right, |l, r| l < r)
    }

    fn compare_less_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        Self::compare_op(left, right, |l, r| l <= r)
    }

    fn compare_greater_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        Self::compare_op(left, right, |l, r| l > r)
    }

    fn compare_greater_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_integer());
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        Self::compare_op(left, right, |l, r| l >= r)
    }

    fn get_data<'a>(&self, _val: &'a Value) -> &'a [u8] {
        panic!("GetData not implemented for TinyintType")
    }

    fn get_storage_size(&self, _val: &Value) -> u32 {
        panic!("GetStorageSize not implemented for TinyintType")
    }
}


