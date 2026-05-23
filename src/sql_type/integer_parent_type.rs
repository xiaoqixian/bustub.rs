//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// integer_parent_type.rs
//
// Identification: src/sql_type/integer_parent_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::sql_type::{CmpBool, SqlType};
use super::type_id::TypeId;
use super::value::Value;

/// An integer value of the common sizes. This trait provides Min/Max
/// implementations, and declares the template arithmetic helper methods.
pub trait IntegerParentType: SqlType {
    /// Default Min implementation for integer types.
    fn min_impl(&self, left: &Value, right: &Value) -> Value {
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

    /// Default Max implementation for integer types.
    fn max_impl(&self, left: &Value, right: &Value) -> Value {
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
}

/// A base integer type that holds a `TypeId`.
pub struct IntegerParentTypeImpl {
    pub type_id: TypeId,
}

impl IntegerParentTypeImpl {
    pub fn new(type_id: TypeId) -> Self {
        IntegerParentTypeImpl { type_id }
    }
}


