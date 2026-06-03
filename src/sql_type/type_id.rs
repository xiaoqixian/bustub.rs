//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// type_id.rs
//
// Identification: src/sql_type/type_id.rs
//
// Copyright (c) 2015-2019, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

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
