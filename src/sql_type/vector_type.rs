//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// vector_type.rs
//
// Identification: src/sql_type/vector_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::limits::*;
use super::sql_type::{CmpBool, SqlType, type_id_to_string};
use super::type_id::TypeId;
use super::value::Value;

/// The SQL VECTOR type (stored as variable-length data of doubles).
pub struct VectorType {
    type_id: TypeId,
}

impl VectorType {
    pub fn new() -> Self {
        VectorType {
            type_id: TypeId::Vector,
        }
    }
}

impl SqlType for VectorType {
    fn get_type_id(&self) -> TypeId { self.type_id }
    fn get_type_size(&self) -> u64 { 0 }
    fn is_coercable_from(&self, type_id: TypeId) -> bool {
        type_id == TypeId::Vector
    }
    fn to_string_id(&self) -> String { type_id_to_string(self.type_id) }
    fn get_min_value(&self) -> Value { Value::from_bytes(TypeId::Vector, &[], 0, false) }
    fn get_max_value(&self) -> Value { Value::from_bytes(TypeId::Vector, &[], 0, false) }

    fn get_data<'a>(&self, _val: &'a Value) -> &'a [u8] {
        unimplemented!("vector type data access not directly supported")
    }

    fn get_storage_size(&self, val: &Value) -> u32 {
        val.size_len
    }

    fn compare_equals(&self, _left: &Value, _right: &Value) -> CmpBool {
        unimplemented!("vector type comparison not supported")
    }

    fn compare_not_equals(&self, _left: &Value, _right: &Value) -> CmpBool {
        unimplemented!("vector type comparison not supported")
    }

    fn compare_less_than(&self, _left: &Value, _right: &Value) -> CmpBool {
        unimplemented!("vector type comparison not supported")
    }

    fn compare_less_than_equals(&self, _left: &Value, _right: &Value) -> CmpBool {
        unimplemented!("vector type comparison not supported")
    }

    fn compare_greater_than(&self, _left: &Value, _right: &Value) -> CmpBool {
        unimplemented!("vector type comparison not supported")
    }

    fn compare_greater_than_equals(&self, _left: &Value, _right: &Value) -> CmpBool {
        unimplemented!("vector type comparison not supported")
    }

    fn add(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type addition not supported")
    }

    fn subtract(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type subtraction not supported")
    }

    fn multiply(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type multiplication not supported")
    }

    fn divide(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type division not supported")
    }

    fn modulo(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type modulo not supported")
    }

    fn min_val(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type min not supported")
    }

    fn max_val(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type max not supported")
    }

    fn sqrt(&self, _val: &Value) -> Value {
        unimplemented!("vector type sqrt not supported")
    }

    fn operate_null(&self, _left: &Value, _right: &Value) -> Value {
        unimplemented!("vector type operate_null not supported")
    }

    fn is_zero(&self, _val: &Value) -> bool {
        unimplemented!("vector type is_zero not supported")
    }

    fn is_inlined(&self, _val: &Value) -> bool {
        false
    }

    fn to_string_val(&self, val: &Value) -> String {
        let len = self.get_storage_size(val);

        if val.is_null() {
            return "vector_null".to_string();
        }
        if len == BUSTUB_VARCHAR_MAX_LEN {
            return "vector_max".to_string();
        }
        if len == 0 {
            return String::new();
        }
        let vec_data = val.get_vector();
        let elems: Vec<String> = vec_data.iter().map(|d| d.to_string()).collect();
        format!("[{}]", elems.join(","))
    }

    fn serialize_to(&self, val: &Value, storage: &mut [u8]) {
        let len = self.get_storage_size(val);
        if len == BUSTUB_VALUE_NULL {
            storage[..4].copy_from_slice(&len.to_le_bytes());
            return;
        }
        storage[..4].copy_from_slice(&len.to_le_bytes());
        let data = val.get_data();
        let copy_len = (len as usize).min(storage.len().saturating_sub(4));
        storage[4..4 + copy_len].copy_from_slice(&data[..copy_len]);
    }

    fn deserialize_from(&self, storage: &[u8]) -> Value {
        let len = u32::from_le_bytes(storage[..4].try_into().unwrap());
        if len == BUSTUB_VALUE_NULL {
            return Value::from_bytes(TypeId::Vector, &[], len, false);
        }
        // Set manage_data as true
        let data = &storage[4..4 + len as usize];
        Value::from_bytes(TypeId::Vector, data, len, true)
    }

    fn copy(&self, val: &Value) -> Value {
        val.clone()
    }

    fn cast_as(&self, _value: &Value, _type_id: TypeId) -> Value {
        unimplemented!("vector type cast not supported")
    }
}


