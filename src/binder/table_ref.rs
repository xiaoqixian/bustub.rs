//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// table_ref.rs
//
// Identification: src/binder/table_ref.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::fmt;

use crate::catalog::{Schema, TableOid};

use super::BoundExpression;
use super::SelectStatement;

//===----------------------------------------------------------------------===//
// JoinType
//===----------------------------------------------------------------------===//

/// All types of joins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    Left,
    Right,
    Inner,
    Outer,
}

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinType::Left => write!(f, "Left"),
            JoinType::Right => write!(f, "Right"),
            JoinType::Inner => write!(f, "Inner"),
            JoinType::Outer => write!(f, "Outer"),
        }
    }
}

//===----------------------------------------------------------------------===//
// TableRef (enum)
//===----------------------------------------------------------------------===//

/// A bound table reference.
pub enum TableRef {
    /// Placeholder for empty FROM clause.
    Empty,
    /// Base table reference, e.g., `SELECT x FROM y`.
    BaseTableRef(BaseTableRef),
    /// Output of cartesian product, e.g., `SELECT * FROM x, y`.
    CrossProductRef(CrossProductRef),
    /// CTE reference, e.g., `WITH (select 1) x SELECT * FROM x`.
    CTERef(CTERef),
    /// Values clause, e.g., `VALUES (1, 2)`.
    ExpressionListRef(ExpressionListRef),
    /// Output of join, e.g., `SELECT * FROM x INNER JOIN y ON ...`.
    JoinRef(JoinRef),
    /// Subquery reference, e.g., `SELECT * FROM (SELECT * FROM t1)`.
    SubqueryRef(SubqueryRef),
}

impl fmt::Display for TableRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableRef::Empty => write!(f, "<empty>"),
            TableRef::BaseTableRef(inner) => fmt::Display::fmt(inner, f),
            TableRef::CrossProductRef(inner) => fmt::Display::fmt(inner, f),
            TableRef::CTERef(inner) => fmt::Display::fmt(inner, f),
            TableRef::ExpressionListRef(inner) => fmt::Display::fmt(inner, f),
            TableRef::JoinRef(inner) => fmt::Display::fmt(inner, f),
            TableRef::SubqueryRef(inner) => fmt::Display::fmt(inner, f),
        }
    }
}

//===----------------------------------------------------------------------===//
// BaseTableRef
//===----------------------------------------------------------------------===//

/// A bound table ref type for single table. e.g., `SELECT x FROM y`,
/// where `y` is `BaseTableRef`.
pub struct BaseTableRef {
    /// The name of the table.
    pub table: String,
    /// The OID of the table.
    pub oid: TableOid,
    /// The alias of the table.
    pub alias: Option<String>,
    /// The schema of the table.
    pub schema: Schema,
}

impl fmt::Display for BaseTableRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(alias) = &self.alias {
            write!(
                f,
                "BaseTableRef {{ table={}, oid={}, alias={} }}",
                self.table, self.oid, alias
            )
        } else {
            write!(
                f,
                "BaseTableRef {{ table={}, oid={} }}",
                self.table, self.oid
            )
        }
    }
}

impl BaseTableRef {
    pub fn get_table_name(&self) -> &str {
        match self.alias.as_ref() {
            Some(a) => a.as_str(),
            None => self.table.as_str()
        }
    }
}

//===----------------------------------------------------------------------===//
// CrossProductRef
//===----------------------------------------------------------------------===//

/// A cross product. e.g., `SELECT * FROM x, y`, where `x, y` is `CrossProductRef`.
pub struct CrossProductRef {
    /// The left side of the cross product.
    pub left: Box<TableRef>,
    /// The right side of the cross product.
    pub right: Box<TableRef>,
}

impl fmt::Display for CrossProductRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CrossProductRef {{ left={}, right={} }}",
            self.left, self.right
        )
    }
}

//===----------------------------------------------------------------------===//
// CTERef
//===----------------------------------------------------------------------===//

/// A CTE. e.g., `WITH (select 1) x SELECT * FROM x`, where `x` is `CTERef`.
pub struct CTERef {
    /// CTE name.
    pub cte_name: String,
    /// Alias.
    pub alias: String,
}

impl fmt::Display for CTERef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CTERef {{ cte_name={}, alias={} }}",
            self.cte_name, self.alias
        )
    }
}

//===----------------------------------------------------------------------===//
// ExpressionListRef
//===----------------------------------------------------------------------===//

/// A bound table ref type for `values` clause.
pub struct ExpressionListRef {
    /// The value list.
    pub values: Vec<Vec<BoundExpression>>,
    /// A unique identifier for this values list, so that planner / binder can work correctly.
    pub identifier: String,
}

impl fmt::Display for ExpressionListRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ExpressionListRef {{ identifier={} }}",
            self.identifier
        )
    }
}

//===----------------------------------------------------------------------===//
// JoinRef
//===----------------------------------------------------------------------===//

/// A join. e.g., `SELECT * FROM x INNER JOIN y ON ...`,
/// where `x INNER JOIN y ON ...` is `JoinRef`.
pub struct JoinRef {
    /// Type of join.
    pub join_type: JoinType,
    /// The left side of the join.
    pub left: Box<TableRef>,
    /// The right side of the join.
    pub right: Box<TableRef>,
    /// Join condition.
    pub condition: BoundExpression,
}

impl fmt::Display for JoinRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Join {{ type={}, left={}, right={}, condition={} }}",
            self.join_type, self.left, self.right, self.condition
        )
    }
}

//===----------------------------------------------------------------------===//
// SubqueryRef
//===----------------------------------------------------------------------===//

/// A subquery. e.g., `SELECT * FROM (SELECT * FROM t1)`,
/// where `(SELECT * FROM t1)` is `SubqueryRef`.
pub struct SubqueryRef {
    /// Subquery.
    pub subquery: SelectStatement,
    /// Name of each item in the select list.
    pub select_list_name: Vec<Vec<String>>,
    /// Alias.
    pub alias: String,
}

impl fmt::Display for SubqueryRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SubqueryRef {{ alias={} }}",
            self.alias
        )
    }
}
