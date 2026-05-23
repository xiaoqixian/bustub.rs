//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// numeric_type.rs
//
// Identification: src/sql_type/numeric_type.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use super::sql_type::SqlType;
use super::type_id::TypeId;

/// Compute the modulo of x and y for floating-point values.
pub fn val_mod(x: f64, y: f64) -> f64 {
    x - (x / y).trunc() * y
}

/// A numeric value is an abstract trait representing a number.
/// Numerics can be either integral or non-integral (decimal), but must
/// provide arithmetic operations on their value.
pub trait NumericType: SqlType {
}

/// A base numeric type that holds a `TypeId`. This is used to avoid
/// repeating boilerplate in numeric type implementations.
pub struct NumericTypeImpl {
    pub type_id: TypeId,
}

impl NumericTypeImpl {
    pub fn new(type_id: TypeId) -> Self {
        NumericTypeImpl { type_id }
    }
}


