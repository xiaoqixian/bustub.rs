mod statement;
mod expression;
mod table_ref;
mod bind_insert;
mod bind_select;

use std::cell::Cell;
use std::fmt;

use sqlparser::ast as sql;
pub use statement::*;
pub use expression::*;
pub use table_ref::*;

use crate::{
    catalog::{Catalog, Column, IndexType, Schema},
    sql_type::{TypeId, Value},
};

//===----------------------------------------------------------------------===//
// BindError
//===----------------------------------------------------------------------===//

/// Errors that can occur during the binding process.
#[derive(Debug)]
pub enum BindError {
    UnsupportedStatement(String),
    /// Unsupported SQL data type encountered.
    UnsupportedDataType(String),
    /// Unsupported table name component encountered.
    UnsupportedTableName(String),
    /// The specified table was not found in the catalog.
    TableNotFound(String),
    /// The specified column was not found in the current scope.
    ColumnNotFound(String),
    /// The specified column reference is ambiguous.
    AmbiguousColumnName(String),
    /// A feature that is not yet implemented.
    NotImplemented(String),
    /// A generic binding exception.
    Exception(String),
    /// An unsupported expression type.
    UnsupportedExpr(String),

    UnsupportedTableFactor(String),
    UnsupportObjectName(String),
    UnsupportedJoinType(String),
    UnsupportedJoinConstraint(String),
    UnsupportedBinaryOperator(String),
    UnsupportedTableRef(String),

    EmptyTableRef,

    UnsupportedYetSQL(String),

    Message(String)
}

impl fmt::Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindError::UnsupportedDataType(ty) => {
                write!(f, "Unsupported data type: {:?}", ty)
            }
            BindError::UnsupportedTableName(name) => {
                write!(f, "Unsupported table name: {:?}", name)
            }
            BindError::TableNotFound(name) => write!(f, "Table not found: {}", name),
            BindError::ColumnNotFound(col) => {
                write!(f, "Column not found: {}", col)
            }
            BindError::AmbiguousColumnName(col) => {
                write!(f, "Ambiguous column: {}", col)
            }
            BindError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            BindError::Exception(msg) => write!(f, "{}", msg),
            BindError::UnsupportedExpr(msg) => write!(f, "Unsupported expression: {}", msg),
            _ => todo!("BindError Display impl")
        }
    }
}

//===----------------------------------------------------------------------===//
// Binder
//===----------------------------------------------------------------------===//

/// The binder is responsible for transforming the sqlparser AST into a binder tree
/// which can be recognized unambiguously by the BusTub planner.
pub struct Binder<'cat> {
    /// Catalog reference used during the binding process.
    catalog: &'cat Catalog,

    /// Universal ID counter for generating unique names for unnamed items.
    #[allow(dead_code)]
    universal_id: Cell<u64>,

    bound_table_ref: Option<TableRef>,
    all_table_ref: Vec<TableRef>
}

#[allow(dead_code)]
impl<'cat> Binder<'cat> {
    pub fn new(catalog: &'cat Catalog) -> Self {
        Self {
            catalog,
            universal_id: Cell::new(0),
            bound_table_ref: None,
            all_table_ref: vec![],
        }
    }

    fn push_table_ref(&mut self, table_ref: TableRef) {
        if let Some(t) = self.bound_table_ref.replace(table_ref) {
           self.all_table_ref.push(t); 
        }
    }

    fn pop_table_ref(&mut self) -> Option<TableRef> {
        if let None = self.bound_table_ref {
            assert!(self.all_table_ref.is_empty());
            return None;
        }
        std::mem::replace(&mut self.bound_table_ref, self.all_table_ref.pop())
    }

    pub fn bind_statement(&mut self, stmt: &sql::Statement) -> Result<BoundStatement, BindError> {
        type Statement = sql::Statement;
        match stmt {
            Statement::Query(query) => self.bind_query(query.as_ref()).map(|s| BoundStatement::Select(s)),
            Statement::CreateTable(ct) => self.bind_create(ct).map(|ct| BoundStatement::Create(ct)),
            Statement::Insert(insert) => self.bind_insert(insert).map(|st| BoundStatement::Insert(st)),
            _ => Err(BindError::UnsupportedStatement(format!("{}", stmt)))
        }
    }
}

#[allow(dead_code)]
impl<'cat> Binder<'cat> {
    /// Generates a unique universal ID for naming unnamed items (e.g., subqueries).
    fn next_id(&self) -> u64 {
        let id = self.universal_id.get();
        self.universal_id.set(id + 1);
        id
    }

    //===----------------------------------------------------------------------===//
    // Column Definition / CREATE TABLE
    //===----------------------------------------------------------------------===//

    fn bind_column_def(&self, col_def: &sql::ColumnDef) -> Result<Column, BindError> {
        match &col_def.data_type {
            // Boolean types
            sql::DataType::Bool | sql::DataType::Boolean => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::Boolean))
            }
            // Tiny integer types
            sql::DataType::TinyInt(_)
            | sql::DataType::TinyIntUnsigned(_)
            | sql::DataType::UTinyInt => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::TinyInt))
            }
            // Small integer types
            sql::DataType::SmallInt(_)
            | sql::DataType::SmallIntUnsigned(_)
            | sql::DataType::USmallInt
            | sql::DataType::Int2(_)
            | sql::DataType::Int2Unsigned(_) => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::SmallInt))
            }
            // Integer types
            sql::DataType::Int(_)
            | sql::DataType::Integer(_)
            | sql::DataType::Int4(_)
            | sql::DataType::IntUnsigned(_)
            | sql::DataType::Int4Unsigned(_)
            | sql::DataType::IntegerUnsigned(_)
            | sql::DataType::Int32
            | sql::DataType::Signed
            | sql::DataType::SignedInteger => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::Integer))
            }
            // Big integer types
            sql::DataType::BigInt(_)
            | sql::DataType::BigIntUnsigned(_)
            | sql::DataType::UBigInt
            | sql::DataType::Int8(_)
            | sql::DataType::Int8Unsigned(_)
            | sql::DataType::Int64 => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::BigInt))
            }
            // Decimal / floating-point types
            sql::DataType::Decimal(_)
            | sql::DataType::Dec(_)
            | sql::DataType::Numeric(_)
            | sql::DataType::Float(_)
            | sql::DataType::Float4
            | sql::DataType::Float8
            | sql::DataType::Float32
            | sql::DataType::Float64
            | sql::DataType::Real
            | sql::DataType::Double(_)
            | sql::DataType::DoublePrecision => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::Decimal))
            }
            // Variable-length character types
            sql::DataType::Varchar(char_len)
            | sql::DataType::Char(char_len)
            | sql::DataType::Character(char_len)
            | sql::DataType::CharacterVarying(char_len)
            | sql::DataType::CharVarying(char_len)
            | sql::DataType::Nvarchar(char_len) => {
                let length = match char_len {
                    Some(sql::CharacterLength::IntegerLength { length, .. }) => *length as usize,
                    _ => 128, // Default length when not specified.
                };
                Ok(Column::new_with_length(
                    col_def.name.value.as_str(),
                    TypeId::Varchar,
                    length,
                ))
            }
            // Timestamp / date types
            sql::DataType::Timestamp { .. }
            | sql::DataType::Date
            | sql::DataType::Datetime { .. } => {
                Ok(Column::new(col_def.name.value.as_str(), TypeId::Timestamp))
            }
            // Unsupported data type
            ty => Err(BindError::UnsupportedDataType(format!("{}", ty))),
        }
    }

    pub fn bind_create(&self, stmt: &sql::CreateTable) -> Result<CreateStatement, BindError> {
        let name_part = stmt.name.0.iter().take(1).next().unwrap();
        let table_name = match name_part {
            sql::ObjectNamePart::Identifier(id) => id.value.clone(),
            x => return Err(BindError::UnsupportedTableName(format!("{}", x))),
        };
        let primary_key = stmt
            .columns
            .iter()
            .map(|x| {
                x.options
                    .iter()
                    .find_map(|y| match y.option {
                        sql::ColumnOption::PrimaryKey(_) => Some(x.name.value.clone()),
                        _ => None,
                    })
            })
            .flatten()
            .collect::<Vec<String>>();
        let columns = stmt
            .columns
            .iter()
            .map(|x| self.bind_column_def(x))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CreateStatement {
            table: table_name,
            columns,
            primary_key,
        })
    }

    pub fn bind_index(&self, stmt: sql::CreateIndex) -> Result<IndexStatement, BindError> {
        // Extract the index name from the statement (e.g., "idx" in CREATE INDEX idx ON t(col)).
        let index_name = match stmt.name {
            Some(name) => {
                let name_part = name.0.into_iter().next().unwrap();
                match name_part {
                    sql::ObjectNamePart::Identifier(id) => id.value,
                    x => return Err(BindError::UnsupportedTableName(format!("{}", x))),
                }
            }
            None => String::new(),
        };

        // Extract the table name from the statement (e.g., "t" in CREATE INDEX idx ON t(col)).
        let table_name_part = stmt.table_name.0.into_iter().next().unwrap();
        let table_name = match table_name_part {
            sql::ObjectNamePart::Identifier(id) => id.value,
            x => return Err(BindError::UnsupportedTableName(format!("{}", x))),
        };

        // Look up the table in the catalog to obtain its OID and schema.
        let table_info = self
            .catalog
            .get_table_by_name(&table_name)
            .ok_or_else(|| BindError::TableNotFound(table_name.clone()))?;

        // Build the BaseTableRef for the target table.
        let table = BaseTableRef {
            table: table_name,
            oid: table_info.oid,
            alias: None,
            schema: Schema::new(table_info.schema.get_columns().clone()),
        };

        // Extract index columns and their sort options (ASC / DESC).
        let mut cols = Vec::new();
        let mut col_options = Vec::new();
        for index_col in stmt.columns {
            let order_by_expr = index_col.column;
            // Extract column name(s) from the expression.
            let col_names = match order_by_expr.expr {
                sql::Expr::Identifier(id) => vec![id.value],
                sql::Expr::CompoundIdentifier(ids) => {
                    ids.into_iter().map(|id| id.value).collect()
                }
                _ => vec![],
            };
            cols.push(BoundColumnRef { col_names });

            // Determine the sort option for this column.
            match order_by_expr.options.asc {
                Some(true) => col_options.push("ASC".to_string()),
                Some(false) => col_options.push("DESC".to_string()),
                None => col_options.push(String::new()),
            }
        }

        // Determine the index type. Default to B+Tree if not specified.
        let index_type = match stmt.using {
            Some(sql::IndexType::BTree) | None => IndexType::BPlusTreeIndex,
            _ => IndexType::BPlusTreeIndex,
        };

        Ok(IndexStatement {
            index_name,
            table,
            cols,
            index_type,
            col_options,
            options: vec![],
        })
    }
}

//===----------------------------------------------------------------------===//
// Bind SELECT Implementation
//===----------------------------------------------------------------------===//

impl<'cat> Binder<'cat> {
    pub fn bind_query(&mut self, query: &sql::Query) -> Result<SelectStatement, BindError> {
        if query.with.is_some() {
            return Err(BindError::UnsupportedYetSQL("Query with WITH is not supported yet".to_owned()));
        }
        if query.order_by.is_some() {
            return Err(BindError::UnsupportedYetSQL("Select with ORDER BY is not supported yet.".to_owned()));
        }
        if query.limit_clause.is_some() {
            return Err(BindError::UnsupportedYetSQL("Select with LIMIT is not supported yet.".to_owned()));
        }
        
        let (table, select_list) = match query.body.as_ref() {
            sql::SetExpr::Select(sel) => {
                (self.bind_from(&sel.from)?, self.bind_select_projection(&sel.projection)?)
            },
            sql::SetExpr::Values(values) => {
                (self.bind_values_list(values)?, vec![])
            }
            x => return Err(BindError::UnsupportedYetSQL(format!("unsupoorted query body: {:?}", x)))
        };

        Ok(SelectStatement {
            table,
            select_list,
            where_clause: None,
            group_by: vec![],
            having: None,
            limit_count: None,
            limit_offset: None,
            sort: vec![],
            ctes: vec![],
            is_distinct: false,
        })
    }

    pub fn bind_expression(&self, expr: &sql::Expr) -> Result<BoundExpression, BindError> {
        type Expr = sql::Expr;
        match expr {
            Expr::Identifier(ident) => {
                let col_names = std::slice::from_ref(&ident.value);
                let table_ref = match self.bound_table_ref.as_ref() {
                    None => return Err(BindError::EmptyTableRef),
                    Some(t) => t
                };
                match Self::resolve_column(table_ref, &col_names)? {
                    Some(expr) => Ok(expr),
                    None => Err(BindError::ColumnNotFound(col_names.join(".")))
                }
            },
            Expr::CompoundIdentifier(idents) => {
                let col_names = idents.iter().map(|i| i.value.clone()).collect::<Vec<_>>();
                let table_ref = match self.bound_table_ref.as_ref() {
                    None => return Err(BindError::EmptyTableRef),
                    Some(t) => t
                };
                match Self::resolve_column(table_ref, &col_names)? {
                    Some(expr) => Ok(expr),
                    None => Err(BindError::ColumnNotFound(col_names.join(".")))
                }
            },
            Expr::BinaryOp {left, op, right} => {
                let larg = Box::new(self.bind_expression(left.as_ref())?);
                let rarg = Box::new(self.bind_expression(right.as_ref())?);
                let op = BoundBinaryOperator::from(op)?;
                Ok(BoundExpression::BoundBinaryOp(BoundBinaryOp {larg, op, rarg}))
            },
            Expr::Value(value_with_span) => self.bind_value(&value_with_span.value).map(|c| BoundExpression::BoundConstant(c)),
            _ => Err(BindError::UnsupportedExpr(format!("{:?}", expr)))
        }
    }

    pub fn bind_value(&self, value: &sql::Value) -> Result<BoundConstant, BindError> {
        match value {
            sql::Value::Number(num_str, false) => {
                let num = num_str.parse::<i32>().map_err(|_| BindError::Message(format!("invalid number literal: {}", num_str)))?;
                let val = Value::from_i32(num);
                Ok(BoundConstant { val })
            },
            sql::Value::SingleQuotedString(s) => Ok(BoundConstant { val: Value::from_str(s.as_str()) }),
            sql::Value::Null => Ok(BoundConstant { val: Value::null(TypeId::Integer) }),
            _ => Err(BindError::Message(format!("unsupported value: {}", value)))
        }
    }

    pub fn bind_from(&self, tables: &Vec<sql::TableWithJoins>) -> Result<TableRef, BindError> {
        match tables.len() {
            0 => Ok(TableRef::Empty),
            1 => self.bind_table_ref(tables.into_iter().next().unwrap()),
            _ => {
                let mut iter = tables.into_iter();
                let mut result = {
                    let l = Box::new(self.bind_table_ref(iter.next().unwrap())?);
                    let r = Box::new(self.bind_table_ref(iter.next().unwrap())?);
                    TableRef::CrossProductRef(CrossProductRef {left: l, right: r})
                };
                for item in iter {
                    let l = Box::new(result);
                    let r = Box::new(self.bind_table_ref(item)?);
                    result = TableRef::CrossProductRef(CrossProductRef {left: l, right: r})
                }
                Ok(result)
            }
        }
    }

    pub fn bind_table_ref(&self, table_with_joins: &sql::TableWithJoins) -> Result<TableRef, BindError> {
        let (table_name, table_alias) = Self::extract_table_fac(&table_with_joins.relation)?;
        let base_table = TableRef::BaseTableRef(self.bind_base_table_ref(table_name, table_alias)?);
        match table_with_joins.joins.len() {
            0 => Ok(base_table),
            _ => {
                table_with_joins.joins.iter()
                    .try_fold(base_table, |b, join| self.bind_table_join(b, join))
            }
        }
    }

    fn bind_values_list(&self, values: &sql::Values) -> Result<TableRef, BindError> {
        bind_select::bind_values_list(&self, values)
    }

    fn bind_select_projection(&self, projection: &Vec<sql::SelectItem>) -> Result<Vec<BoundExpression>, BindError> {
        let table_ref = match self.bound_table_ref.as_ref() {
            Some(t) => t,
            None => return Err(BindError::Message("Empty table ref".to_string()))
        };

        let mut select_list = Vec::new();
        let mut is_select_star = false;
        let mut has_agg= false;
        let mut has_window= false;
        for proj in projection {
            let expr = match proj {
                sql::SelectItem::UnnamedExpr(expr) => self.bind_expression(expr)?,
                sql::SelectItem::Wildcard(_) => BoundExpression::BoundStar,
                _ => return Err(BindError::UnsupportedExpr(format!("{}", proj)))
            };

            match expr {
                BoundExpression::BoundStar => {
                    if !select_list.is_empty() {
                        return Err(BindError::Message("select * cannot have other expressions in list".to_string()));
                    }
                    select_list = Self::get_all_columns(table_ref)?;
                    is_select_star = true;
                },
                expr => {
                    if is_select_star {
                        return Err(BindError::Message("select * cannot have other expressions in list".to_string()));
                    }
                    if expr.has_aggregation() { has_agg = true; }
                    if expr.has_window_function() { has_window = true; }
                    select_list.push(expr);
                }
            }
        }
        if has_window && has_agg {
            return Err(BindError::Message("cannot have both normal agg and window agg in same query".to_string()));
        }

        Ok(select_list)
    }

    fn get_all_columns(table_ref: &TableRef) -> Result<Vec<BoundExpression>, BindError> {
        match table_ref {
            TableRef::BaseTableRef(tr) => {
                Ok(tr.schema.columns.iter()
                    .map(|c| BoundExpression::BoundColumnRef(BoundColumnRef {col_names: vec![tr.table.clone(), c.column_name.clone()]}))
                    .collect())
            },
            TableRef::CrossProductRef(cr) => {
                let mut columns = Self::get_all_columns(&cr.left)?;
                let append_columns = Self::get_all_columns(&cr.right)?;
                columns.extend(append_columns);
                Ok(columns)
            },
            TableRef::JoinRef(jr) => {
                let mut columns = Self::get_all_columns(&jr.left)?;
                let append_columns = Self::get_all_columns(&jr.right)?;
                columns.extend(append_columns);
                Ok(columns)
            }
            _ => Err(BindError::Message(format!("select * cannot be used with table ref {}", table_ref)))
        }
    }

    pub fn bind_base_table_ref(&self, table_name: String, table_alias: Option<String>) -> Result<BaseTableRef, BindError> {
        match self.catalog.get_table_by_name(&table_name).as_ref() {
            None => Err(BindError::TableNotFound(table_name)),
            Some(table_info) => Ok(
                BaseTableRef {
                    table: table_name,
                    oid: table_info.oid,
                    alias: table_alias,
                    schema: table_info.schema.clone()
                }
            )
        }
    }

    fn resolve_column(table_ref: &TableRef, col_name: &[String]) -> Result<Option<BoundExpression>, BindError> {
        match table_ref {
            TableRef::BaseTableRef(base_table_ref) => {
                Self::resolve_column_ref_from_base_table_ref(base_table_ref, col_name)
                    .map(|x| x.map(|y| BoundExpression::BoundColumnRef(y)))
            },
            TableRef::CrossProductRef(cross_product_ref) => {
                match Self::resolve_column(cross_product_ref.left.as_ref(), col_name)? {
                    Some(x) => Ok(Some(x)),
                    None => Self::resolve_column(cross_product_ref.right.as_ref(), col_name)
                }
            },
            TableRef::JoinRef(join_ref) => {
                match Self::resolve_column(join_ref.left.as_ref(), col_name)? {
                    Some(x) => Ok(Some(x)),
                    None => Self::resolve_column(join_ref.right.as_ref(), col_name)
                }
            },
            _ => Err(BindError::UnsupportedTableRef(format!("{}", table_ref)))
        }
    }

    fn resolve_column_ref_from_base_table_ref(table_ref: &BaseTableRef, col_names: &[String]) -> Result<Option<BoundColumnRef>, BindError> {
        let table_name = table_ref.get_table_name();
        let col_name = match col_names.len() {
            1 => {
                col_names[0].as_str()
            },
            _ => {
                if col_names[0].as_str() == table_name {
                    col_names[1].as_str()
                } else {
                    return Ok(None);
                }
            }
        };
        if Self::resolve_column_name_from_schema(&table_ref.schema, col_name)? {
            Ok(Some(BoundColumnRef {
                col_names: vec![table_name.to_owned(), col_name.to_owned()]
            }))
        } else {
            Ok(None)
        }
    }

    fn resolve_column_name_from_schema(schema: &Schema, col_name: &str) -> Result<bool, BindError> {
        let mut found = false;
        for col in schema.columns.iter() {
            if Self::eq_ignore_case(col_name, col.get_name()) {
                if found {
                    return Err(BindError::AmbiguousColumnName(col_name.to_owned()));
                }
                found = true;
            }
        }
        Ok(found)
    }

    fn bind_table_join(&self, l_table: TableRef, join: &sql::Join) -> Result<TableRef, BindError> {
        let (r_table_name, r_table_alias) = Self::extract_table_fac(&join.relation)?;
        let r_table = Box::new(TableRef::BaseTableRef(self.bind_base_table_ref(r_table_name, r_table_alias)?));
        let (join_type, constraint) = match &join.join_operator {
            sql::JoinOperator::Left(c) => (JoinType::Left, c),
            sql::JoinOperator::Right(c) => (JoinType::Right, c),
            sql::JoinOperator::Inner(c) => (JoinType::Inner, c),
            sql::JoinOperator::FullOuter(c) => (JoinType::Outer, c),
            _ => return Err(BindError::UnsupportedJoinType(format!("{:?}", join.join_operator)))
        };
        
        let condition = match constraint {
            sql::JoinConstraint::On(expr) => self.bind_expression(&expr)?,
            _ => return Err(BindError::UnsupportedJoinConstraint(format!("{:?}", constraint)))
        };
        Ok(TableRef::JoinRef(JoinRef {
            left: Box::new(l_table),
            right: r_table,
            join_type,
            condition
        }))
    }

    fn extract_table_fac(table_fac: &sql::TableFactor) -> Result<(String, Option<String>), BindError> {
        match table_fac {
            sql::TableFactor::Table {
                name,
                alias,
                ..
            } => {
                Ok((Self::extract_object_name(&name)?, alias.as_ref().map(|x| x.name.value.clone())))
            },
            _ => Err(BindError::UnsupportedTableFactor(format!("{:?}", table_fac)))
        }
    }

    fn extract_object_name(name: &sql::ObjectName) -> Result<String, BindError> {
        if name.0.is_empty() || name.0.len() > 1 {
            return Err(BindError::UnsupportObjectName(format!("{:?}", name)));
        }
        match &name.0[0] {
            sql::ObjectNamePart::Identifier(ident) => Ok(ident.value.clone()),
            _ => Err(BindError::UnsupportObjectName(format!("{:?}", name)))
        }
    }

    fn eq_ignore_case(a: &str, b: &str) -> bool {
        let a_iter = a.chars().flat_map(|c| c.to_lowercase());
        let b_iter = b.chars().flat_map(|c| c.to_lowercase());
        a_iter.eq(b_iter)
    }
}

// bind insert
impl<'a> Binder<'a> {
    fn bind_insert(&mut self, insert: &sql::Insert) -> Result<InsertStatement, BindError> {
        bind_insert::bind_insert(self, insert)
    }
}
