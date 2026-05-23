//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// varlen_type.rs
//
// Identification: src/sql_type/varlen_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::sql_type::{get_cmp_bool, CmpBool, SqlType, type_id_to_string};
use super::type_id::TypeId;
use super::type_util::TypeUtil;
use super::value::Value;
use std::cmp::Ordering;

/// A variable-length value type representing all objects that have variable
/// length (e.g., VARCHAR).
pub struct VarlenType {
    type_id: TypeId,
}

impl VarlenType {
    pub fn new(type_id: TypeId) -> Self {
        VarlenType { type_id }
    }
}

impl SqlType for VarlenType {
    fn get_type_id(&self) -> TypeId { self.type_id }
    fn get_type_size(&self) -> u64 { 0 }
    fn is_coercable_from(&self, type_id: TypeId) -> bool {
        matches!(
            type_id,
            TypeId::Boolean
                | TypeId::TinyInt
                | TypeId::SmallInt
                | TypeId::Integer
                | TypeId::BigInt
                | TypeId::Decimal
                | TypeId::Timestamp
                | TypeId::Varchar
        )
    }
    fn to_string_id(&self) -> String { type_id_to_string(self.type_id) }
    fn get_min_value(&self) -> Value { Value::from_string(TypeId::Varchar, "") }
    fn get_max_value(&self) -> Value { Value::from_bytes(TypeId::Varchar, &[], 0, false) }

    fn get_data<'a>(&self, val: &'a Value) -> &'a [u8] {
        match val.raw_value {
            super::value::ValuePayload::Varlen(ref data) => data.as_slice(),
            _ => panic!("GetData called on non-varlen value"),
        }
    }

    fn get_storage_size(&self, val: &Value) -> u32 {
        val.size_len
    }

    fn compare_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        let str1 = self.get_data(left);
        let len1 = self.get_storage_size(left).saturating_sub(1) as usize;
        let r_value;
        let (str2, len2) = if right.get_type_id() == TypeId::Varchar {
            (self.get_data(right), self.get_storage_size(right).saturating_sub(1) as usize)
        } else {
            r_value = right.cast_as(TypeId::Varchar);
            (self.get_data(&r_value), self.get_storage_size(&r_value).saturating_sub(1) as usize)
        };
        get_cmp_bool(TypeUtil::compare_strings(str1, len1, str2, len2) == Ordering::Equal)
    }

    fn compare_not_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        let str1 = self.get_data(left);
        let len1 = self.get_storage_size(left).saturating_sub(1) as usize;
        let r_value;
        let (str2, len2) = if right.get_type_id() == TypeId::Varchar {
            (self.get_data(right), self.get_storage_size(right).saturating_sub(1) as usize)
        } else {
            r_value = right.cast_as(TypeId::Varchar);
            (self.get_data(&r_value), self.get_storage_size(&r_value).saturating_sub(1) as usize)
        };
        get_cmp_bool(TypeUtil::compare_strings(str1, len1, str2, len2) != Ordering::Equal)
    }

    fn compare_less_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        let str1 = self.get_data(left);
        let len1 = self.get_storage_size(left).saturating_sub(1) as usize;
        let r_value;
        let (str2, len2) = if right.get_type_id() == TypeId::Varchar {
            (self.get_data(right), self.get_storage_size(right).saturating_sub(1) as usize)
        } else {
            r_value = right.cast_as(TypeId::Varchar);
            (self.get_data(&r_value), self.get_storage_size(&r_value).saturating_sub(1) as usize)
        };
        get_cmp_bool(TypeUtil::compare_strings(str1, len1, str2, len2) == Ordering::Less)
    }

    fn compare_less_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        let str1 = self.get_data(left);
        let len1 = self.get_storage_size(left).saturating_sub(1) as usize;
        let r_value;
        let (str2, len2) = if right.get_type_id() == TypeId::Varchar {
            (self.get_data(right), self.get_storage_size(right).saturating_sub(1) as usize)
        } else {
            r_value = right.cast_as(TypeId::Varchar);
            (self.get_data(&r_value), self.get_storage_size(&r_value).saturating_sub(1) as usize)
        };
        get_cmp_bool(
            TypeUtil::compare_strings(str1, len1, str2, len2) != Ordering::Greater,
        )
    }

    fn compare_greater_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        let str1 = self.get_data(left);
        let len1 = self.get_storage_size(left).saturating_sub(1) as usize;
        let r_value;
        let (str2, len2) = if right.get_type_id() == TypeId::Varchar {
            (self.get_data(right), self.get_storage_size(right).saturating_sub(1) as usize)
        } else {
            r_value = right.cast_as(TypeId::Varchar);
            (self.get_data(&r_value), self.get_storage_size(&r_value).saturating_sub(1) as usize)
        };
        get_cmp_bool(TypeUtil::compare_strings(str1, len1, str2, len2) == Ordering::Greater)
    }

    fn compare_greater_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        let str1 = self.get_data(left);
        let len1 = self.get_storage_size(left).saturating_sub(1) as usize;
        let r_value;
        let (str2, len2) = if right.get_type_id() == TypeId::Varchar {
            (self.get_data(right), self.get_storage_size(right).saturating_sub(1) as usize)
        } else {
            r_value = right.cast_as(TypeId::Varchar);
            (self.get_data(&r_value), self.get_storage_size(&r_value).saturating_sub(1) as usize)
        };
        get_cmp_bool(
            TypeUtil::compare_strings(str1, len1, str2, len2) != Ordering::Less,
        )
    }

    fn add(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Add not implemented for VarlenType")
    }
    fn subtract(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Subtract not implemented for VarlenType")
    }
    fn multiply(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Multiply not implemented for VarlenType")
    }
    fn divide(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Divide not implemented for VarlenType")
    }
    fn modulo(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Modulo not implemented for VarlenType")
    }

    fn min_val(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return left.operate_null(right);
        }
        if left.compare_less_than(right) == CmpBool::CmpTrue {
            self.copy(left)
        } else {
            self.copy(right)
        }
    }

    fn max_val(&self, left: &Value, right: &Value) -> Value {
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return left.operate_null(right);
        }
        if left.compare_greater_than(right) == CmpBool::CmpTrue {
            self.copy(left)
        } else {
            self.copy(right)
        }
    }

    fn sqrt(&self, _val: &Value) -> Value {
        panic!("Sqrt not implemented for VarlenType")
    }

    fn operate_null(&self, _left: &Value, _right: &Value) -> Value {
        Value::from_bytes(TypeId::Varchar, &[], BUSTUB_VALUE_NULL, false)
    }

    fn is_zero(&self, _val: &Value) -> bool {
        panic!("IsZero not implemented for VarlenType")
    }

    fn is_inlined(&self, _val: &Value) -> bool {
        false
    }

    fn to_string_val(&self, val: &Value) -> String {
        let len = self.get_storage_size(val);

        if val.is_null() {
            return "varlen_null".to_string();
        }
        if len == BUSTUB_VARCHAR_MAX_LEN {
            return "varlen_max".to_string();
        }
        if len == 0 {
            return String::new();
        }
        let data = self.get_data(val);
        let text_len = (len.saturating_sub(1)) as usize;
        if text_len > data.len() {
            return String::new();
        }
        String::from_utf8_lossy(&data[..text_len]).to_string()
    }

    fn serialize_to(&self, val: &Value, storage: &mut [u8]) {
        let len = self.get_storage_size(val);
        if len == BUSTUB_VALUE_NULL {
            storage[..4].copy_from_slice(&len.to_le_bytes());
            return;
        }
        storage[..4].copy_from_slice(&len.to_le_bytes());
        let data = self.get_data(val);
        let copy_len = (len as usize).min(storage.len().saturating_sub(4));
        storage[4..4 + copy_len].copy_from_slice(&data[..copy_len]);
    }

    fn deserialize_from(&self, storage: &[u8]) -> Value {
        let len = u32::from_le_bytes(storage[..4].try_into().unwrap());
        if len == BUSTUB_VALUE_NULL {
            return Value::from_bytes(TypeId::Varchar, &[], len, false);
        }
        // Set manage_data as true
        let data = &storage[4..4 + len as usize];
        Value::from_bytes(TypeId::Varchar, data, len, true)
    }

    fn copy(&self, val: &Value) -> Value {
        val.clone()
    }

    fn cast_as(&self, value: &Value, type_id: TypeId) -> Value {
        let str_val = value.to_string_val();
        match type_id {
            TypeId::Boolean => {
                let s = str_val.to_lowercase();
                if s == "true" || s == "1" || s == "t" {
                    return Value::from_i8(TypeId::Boolean, 1);
                }
                if s == "false" || s == "0" || s == "f" {
                    return Value::from_i8(TypeId::Boolean, 0);
                }
                panic!("Boolean value format error.");
            }
            TypeId::TinyInt => {
                let v: i8 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                if v < BUSTUB_INT8_MIN {
                    panic!("Numeric value out of range.");
                }
                Value::from_i8(TypeId::TinyInt, v)
            }
            TypeId::SmallInt => {
                let v: i16 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                if v < BUSTUB_INT16_MIN {
                    panic!("Numeric value out of range.");
                }
                Value::from_i16(TypeId::SmallInt, v)
            }
            TypeId::Integer => {
                let v: i32 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                if v > BUSTUB_INT32_MAX || v < BUSTUB_INT32_MIN {
                    panic!("Numeric value out of range.");
                }
                Value::from_i32(TypeId::Integer, v)
            }
            TypeId::BigInt => {
                let v: i64 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                if v > BUSTUB_INT64_MAX || v < BUSTUB_INT64_MIN {
                    panic!("Numeric value out of range.");
                }
                Value::from_i64(TypeId::BigInt, v)
            }
            TypeId::Decimal => {
                let v: f64 = str_val
                    .parse()
                    .unwrap_or_else(|_| panic!("Numeric value out of range."));
                if v > BUSTUB_DECIMAL_MAX || v < BUSTUB_DECIMAL_MIN {
                    panic!("Numeric value out of range.");
                }
                Value::from_f64(TypeId::Decimal, v)
            }
            TypeId::Varchar => value.copy_val(),
            _ => {
                panic!(
                    "VARCHAR is not coercable to {}",
                    type_id_to_string(type_id)
                )
            }
        }
    }
}


