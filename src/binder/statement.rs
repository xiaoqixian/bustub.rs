//===----------------------------------------------------------------------===//
//
//                         BusTub
//
// statement.rs
//
// Identification: src/binder/statement.rs
//
// Copyright (c) 2015-2025, Carnegie Mellon University Database Group
//
//===----------------------------------------------------------------------===//

use std::fmt;

use crate::catalog::Column;
use crate::catalog::IndexType;

use super::BaseTableRef;
use super::BoundColumnRef;
use super::BoundExpression;
use super::BoundOrderBy;
use super::TableRef;

//===----------------------------------------------------------------------===//
// ExplainOptions
//===----------------------------------------------------------------------===//

/// Options for EXPLAIN statement, used as bit flags.
pub enum ExplainOptions {
    /// Show binder results.
    Binder = 1,
    /// Show planner results.
    Planner = 2,
    /// Show optimizer results.
    Optimizer = 4,
    /// Show schema.
    Schema = 8,
}

//===----------------------------------------------------------------------===//
// Statement (enum)
//===----------------------------------------------------------------------===//

/// A bound SQL statement.
pub enum BoundStatement {
    /// CREATE TABLE statement.
    Create(CreateStatement),
    /// SELECT statement.
    Select(SelectStatement),
    /// DELETE statement.
    Delete(DeleteStatement),
    /// INSERT statement.
    Insert(InsertStatement),
    /// EXPLAIN statement.
    Explain(ExplainStatement),
    /// UPDATE statement.
    Update(UpdateStatement),
    /// SET a system variable.
    VariableSet(VariableSetStatement),
    /// SHOW a system variable.
    VariableShow(VariableShowStatement),
    /// Transaction control statement.
    Transaction(TransactionStatement),
    /// CREATE INDEX statement.
    Index(IndexStatement),
}

impl fmt::Display for BoundStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundStatement::Create(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Select(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Delete(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Insert(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Explain(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Update(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::VariableSet(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::VariableShow(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Transaction(inner) => fmt::Display::fmt(inner, f),
            BoundStatement::Index(inner) => fmt::Display::fmt(inner, f),
        }
    }
}

//===----------------------------------------------------------------------===//
// CreateStatement
//===----------------------------------------------------------------------===//

/// A bound CREATE TABLE statement.
pub struct CreateStatement {
    /// The name of the table to create.
    pub table: String,
    /// The columns of the table.
    pub columns: Vec<Column>,
    /// The primary key column names.
    pub primary_key: Vec<String>,
}

impl fmt::Display for CreateStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CreateStatement {{ table={} }}", self.table)
    }
}

//===----------------------------------------------------------------------===//
// SelectStatement
//===----------------------------------------------------------------------===//

/// A bound SELECT statement.
pub struct SelectStatement {
    /// Bound FROM clause.
    pub table: Box<TableRef>,
    /// Bound SELECT list.
    pub select_list: Vec<BoundExpression>,
    /// Bound WHERE clause.
    pub where_clause: Option<BoundExpression>,
    /// Bound GROUP BY clause.
    pub group_by: Vec<BoundExpression>,
    /// Bound HAVING clause.
    pub having: Option<BoundExpression>,
    /// Bound LIMIT count.
    pub limit_count: Option<BoundExpression>,
    /// Bound LIMIT offset.
    pub limit_offset: Option<BoundExpression>,
    /// Bound ORDER BY clause.
    pub sort: Vec<BoundOrderBy>,
    /// Bound CTE list.
    pub ctes: Vec<super::SubqueryRef>,
    /// Is SELECT DISTINCT.
    pub is_distinct: bool,
}

impl fmt::Display for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SelectStatement {{ table={} }}", self.table)
    }
}

//===----------------------------------------------------------------------===//
// DeleteStatement
//===----------------------------------------------------------------------===//

/// A bound DELETE statement.
pub struct DeleteStatement {
    /// The table to delete from.
    pub table: BaseTableRef,
    /// The filter expression (WHERE clause).
    pub expr: Option<BoundExpression>,
}

impl fmt::Display for DeleteStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeleteStatement {{ table={} }}", self.table)
    }
}

//===----------------------------------------------------------------------===//
// InsertStatement
//===----------------------------------------------------------------------===//

/// A bound INSERT statement.
pub struct InsertStatement {
    /// The table to insert into.
    pub table: BaseTableRef,
    /// The SELECT statement providing values to insert.
    pub select: SelectStatement,
}

impl fmt::Display for InsertStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InsertStatement {{ table={} }}", self.table)
    }
}

//===----------------------------------------------------------------------===//
// ExplainStatement
//===----------------------------------------------------------------------===//

/// A bound EXPLAIN statement.
pub struct ExplainStatement {
    /// The inner statement being explained.
    pub statement: Box<BoundStatement>,
    /// Explain options as bit flags.
    pub options: u8,
}

impl fmt::Display for ExplainStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExplainStatement")
    }
}

//===----------------------------------------------------------------------===//
// UpdateStatement
//===----------------------------------------------------------------------===//

/// A bound UPDATE statement.
pub struct UpdateStatement {
    /// The table to update.
    pub table: BaseTableRef,
    /// The filter expression (WHERE clause).
    pub filter_expr: Option<BoundExpression>,
    /// The target column and expression pairs to update.
    pub target_expr: Vec<(BoundColumnRef, BoundExpression)>,
}

impl fmt::Display for UpdateStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UpdateStatement {{ table={} }}", self.table)
    }
}

//===----------------------------------------------------------------------===//
// VariableSetStatement
//===----------------------------------------------------------------------===//

/// A bound SET statement, e.g., `SET variable = value`.
pub struct VariableSetStatement {
    /// The variable name.
    pub variable: String,
    /// The value to set.
    pub value: String,
}

impl fmt::Display for VariableSetStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VariableSet {{ variable={}, value={} }}",
            self.variable, self.value
        )
    }
}

//===----------------------------------------------------------------------===//
// VariableShowStatement
//===----------------------------------------------------------------------===//

/// A bound SHOW statement, e.g., `SHOW variable`.
pub struct VariableShowStatement {
    /// The variable name.
    pub variable: String,
}

impl fmt::Display for VariableShowStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VariableShow {{ variable={} }}", self.variable)
    }
}

//===----------------------------------------------------------------------===//
// TransactionStatement
//===----------------------------------------------------------------------===//

#[derive(Debug, Clone)]
pub enum TransactionStatementType {
    Begin,
    Commit,
    Rollback
}
/// A bound transaction control statement, e.g., `BEGIN`, `COMMIT`, `ROLLBACK`.
pub struct TransactionStatement {
    /// The transaction command type.
    pub txn_type: TransactionStatementType,
}

impl fmt::Display for TransactionStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.txn_type {
            TransactionStatementType::Begin => write!(f, "BEGIN"),
            TransactionStatementType::Commit => write!(f, "COMMIT"),
            TransactionStatementType::Rollback => write!(f, "ROLLBACK"),
        }
    }
}

//===----------------------------------------------------------------------===//
// IndexStatement
//===----------------------------------------------------------------------===//

/// A bound CREATE INDEX statement.
pub struct IndexStatement {
    /// The name of the index.
    pub index_name: String,
    /// The table on which the index is created.
    pub table: BaseTableRef,
    /// The columns to index.
    pub cols: Vec<BoundColumnRef>,
    /// The index type (e.g., "btree", "hash").
    pub index_type: IndexType,
    /// Column-level options.
    pub col_options: Vec<String>,
    /// Additional options as key-value pairs.
    pub options: Vec<(String, i32)>,
}

impl fmt::Display for IndexStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IndexStatement {{ index_name={}, table={} }}",
            self.index_name, self.table
        )
    }
}
