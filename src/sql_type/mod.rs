//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// mod.rs
//
// Identification: src/sql_type/mod.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::fmt::Display;

pub mod value;
pub mod limits;
pub use value::Value;
/// Every possible SQL type ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeId {
    Boolean,
    TinyInt,
    SmallInt,
    Integer,
    BigInt,
    Decimal,
    Varchar,
    Timestamp,
}

/// Comparison result enum that supports three-valued logic (SQL NULL semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpBool {
    CmpFalse = 0,
    CmpTrue = 1,
    CmpNull = 2,
}

impl From<bool> for CmpBool {
    fn from(value: bool) -> Self {
        match value {
            true => CmpBool::CmpTrue,
            false => CmpBool::CmpFalse,
        }
    }
}

impl Display for CmpBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmpBool::CmpFalse => write!(f, "false"),
            CmpBool::CmpTrue => write!(f, "true"),
            CmpBool::CmpNull => write!(f, "null"),
        }
    }
}


impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeId::Boolean => write!(f, "Boolean"),
            TypeId::TinyInt => write!(f, "TinyInt"),
            TypeId::SmallInt => write!(f, "SmallInt"),
            TypeId::Integer => write!(f, "Integer"),
            TypeId::BigInt => write!(f, "BigInt"),
            TypeId::Decimal => write!(f, "Decimal"),
            TypeId::Varchar => write!(f, "Varchar"),
            TypeId::Timestamp => write!(f, "Timestamp"),
        }
    }
}
