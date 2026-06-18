//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// boolean_type.rs
//
// Identification: src/sql_type/boolean_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::sql_type::{get_cmp_bool, CmpBool, SqlType, type_id_to_string};
use super::type_id::TypeId;
use super::value::Value;

/// A boolean value isn't a real SQL type, but we treat it as one to keep
/// consistent in the expression subsystem.
pub struct BooleanType {
    type_id: TypeId,
}

impl BooleanType {
    pub fn new() -> Self {
        BooleanType {
            type_id: TypeId::Boolean,
        }
    }
}

impl SqlType for BooleanType {
    fn get_type_id(&self) -> TypeId {
        self.type_id
    }

    fn get_type_size(&self) -> u64 {
        1
    }

    fn is_coercable_from(&self, _type_id: TypeId) -> bool {
        true
    }

    fn to_string_id(&self) -> String {
        type_id_to_string(self.type_id)
    }

    fn get_min_value(&self) -> Value {
        Value::from_i8(TypeId::Boolean, 0)
    }

    fn get_max_value(&self) -> Value {
        Value::from_i8(TypeId::Boolean, 1)
    }

    fn compare_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Boolean);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        get_cmp_bool(
            left.get_as::<i8>() == right.cast_as(TypeId::Boolean).get_as::<i8>(),
        )
    }

    fn compare_not_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Boolean);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        get_cmp_bool(
            left.get_as::<i8>() != right.cast_as(TypeId::Boolean).get_as::<i8>(),
        )
    }

    fn compare_less_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Boolean);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        get_cmp_bool(
            left.get_as::<i8>() < right.cast_as(TypeId::Boolean).get_as::<i8>(),
        )
    }

    fn compare_less_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Boolean);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        get_cmp_bool(
            left.get_as::<i8>() <= right.cast_as(TypeId::Boolean).get_as::<i8>(),
        )
    }

    fn compare_greater_than(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Boolean);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        get_cmp_bool(
            left.get_as::<i8>() > right.cast_as(TypeId::Boolean).get_as::<i8>(),
        )
    }

    fn compare_greater_than_equals(&self, left: &Value, right: &Value) -> CmpBool {
        assert!(self.get_type_id() == TypeId::Boolean);
        assert!(left.check_comparable(right));
        if left.is_null() || right.is_null() {
            return CmpBool::CmpNull;
        }
        get_cmp_bool(
            left.get_as::<i8>() >= right.cast_as(TypeId::Boolean).get_as::<i8>(),
        )
    }

    fn add(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Add not implemented for BooleanType")
    }

    fn subtract(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Subtract not implemented for BooleanType")
    }

    fn multiply(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Multiply not implemented for BooleanType")
    }

    fn divide(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Divide not implemented for BooleanType")
    }

    fn modulo(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Modulo not implemented for BooleanType")
    }

    fn min_val(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Min not implemented for BooleanType")
    }

    fn max_val(&self, _left: &Value, _right: &Value) -> Value {
        panic!("Max not implemented for BooleanType")
    }

    fn sqrt(&self, _val: &Value) -> Value {
        panic!("Sqrt not implemented for BooleanType")
    }

    fn operate_null(&self, _left: &Value, _right: &Value) -> Value {
        panic!("OperateNull not implemented for BooleanType")
    }

    fn is_zero(&self, _val: &Value) -> bool {
        panic!("IsZero not implemented for BooleanType")
    }

    fn is_inlined(&self, _val: &Value) -> bool {
        true
    }

    fn to_string_val(&self, val: &Value) -> String {
        assert!(self.get_type_id() == TypeId::Boolean);
        let b = val.get_as::<i8>();
        if b == 1 {
            "true".to_string()
        } else if b == 0 {
            "false".to_string()
        } else {
            "boolean_null".to_string()
        }
    }

    fn serialize_to(&self, val: &Value, storage: &mut [u8]) {
        let v: i8 = val.get_as();
        storage[0..1].copy_from_slice(&v.to_le_bytes());
    }

    fn deserialize_from(&self, storage: &[u8]) -> Value {
        let val = i8::from_le_bytes(storage[..1].try_into().unwrap());
        Value::from_i8(TypeId::Boolean, val)
    }

    fn copy(&self, val: &Value) -> Value {
        Value::from_i8(TypeId::Boolean, val.get_as::<i8>())
    }

    fn cast_as(&self, val: &Value, type_id: TypeId) -> Value {
        match type_id {
            TypeId::Boolean => self.copy(val),
            TypeId::Varchar => {
                if val.is_null() {
                    return Value::from_bytes(TypeId::Varchar, &[], 0, false);
                }
                Value::from_str(&val.to_string_val())
            }
            _ => {
                panic!(
                    "BOOLEAN is not coercable to {}",
                    type_id_to_string(type_id)
                )
            }
        }
    }

    fn get_data<'a>(&self, _val: &'a Value) -> &'a [u8] {
        panic!("GetData not implemented for BooleanType")
    }

    fn get_storage_size(&self, _val: &Value) -> u32 {
        panic!("GetStorageSize not implemented for BooleanType")
    }
}


