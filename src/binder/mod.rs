mod statement;
mod expression;
mod table_ref;

use sqlparser::ast as sql;
pub use statement::*;
pub use expression::*;
pub use table_ref::*;

use crate::{catalog::{Catalog, Column, IndexType, Schema}, sql_type::TypeId};

pub enum BindError {
    UnsupportedDataType(sql::DataType),
    UnsupportedTableName(sql::ObjectNamePart),
    TableNotFound(String),
}

pub struct Binder<'cat> {
    catalog: &'cat Catalog,
}

impl<'cat> Binder<'cat> {
    fn bind_column_def(&self, col_def: sql::ColumnDef) -> Result<Column, BindError> {
        match col_def.data_type {
            // Boolean types
            sql::DataType::Bool | sql::DataType::Boolean => {
                Ok(Column::new(col_def.name.value, TypeId::Boolean))
            }
            // Tiny integer types
            sql::DataType::TinyInt(_) | sql::DataType::TinyIntUnsigned(_)
            | sql::DataType::UTinyInt => {
                Ok(Column::new(col_def.name.value, TypeId::TinyInt))
            }
            // Small integer types
            sql::DataType::SmallInt(_) | sql::DataType::SmallIntUnsigned(_)
            | sql::DataType::USmallInt | sql::DataType::Int2(_)
            | sql::DataType::Int2Unsigned(_) => {
                Ok(Column::new(col_def.name.value, TypeId::SmallInt))
            }
            // Integer types
            sql::DataType::Int(_) | sql::DataType::Integer(_)
            | sql::DataType::Int4(_) | sql::DataType::IntUnsigned(_)
            | sql::DataType::Int4Unsigned(_) | sql::DataType::IntegerUnsigned(_)
            | sql::DataType::Int32 | sql::DataType::Signed
            | sql::DataType::SignedInteger => {
                Ok(Column::new(col_def.name.value, TypeId::Integer))
            }
            // Big integer types
            sql::DataType::BigInt(_) | sql::DataType::BigIntUnsigned(_)
            | sql::DataType::UBigInt | sql::DataType::Int8(_)
            | sql::DataType::Int8Unsigned(_) | sql::DataType::Int64 => {
                Ok(Column::new(col_def.name.value, TypeId::BigInt))
            }
            // Decimal / floating-point types
            sql::DataType::Decimal(_) | sql::DataType::Dec(_)
            | sql::DataType::Numeric(_) | sql::DataType::Float(_)
            | sql::DataType::Float4 | sql::DataType::Float8
            | sql::DataType::Float32 | sql::DataType::Float64
            | sql::DataType::Real | sql::DataType::Double(_)
            | sql::DataType::DoublePrecision => {
                Ok(Column::new(col_def.name.value, TypeId::Decimal))
            }
            // Variable-length character types
            sql::DataType::Varchar(char_len) | sql::DataType::Char(char_len)
            | sql::DataType::Character(char_len)
            | sql::DataType::CharacterVarying(char_len)
            | sql::DataType::CharVarying(char_len)
            | sql::DataType::Nvarchar(char_len) => {
                let length = match char_len {
                    Some(sql::CharacterLength::IntegerLength { length, .. }) => length as u32,
                    _ => 128, // default length when not specified
                };
                Ok(Column::new_with_length(col_def.name.value, TypeId::Varchar, length))
            }
            // Timestamp / date types
            sql::DataType::Timestamp { .. } | sql::DataType::Date
            | sql::DataType::Datetime { .. } => {
                Ok(Column::new(col_def.name.value, TypeId::Timestamp))
            }
            // Unsupported data type
            ty => Err(BindError::UnsupportedDataType(ty)),
        }
    }

    pub fn bind_create(&self, stmt: sql::CreateTable) -> Result<CreateStatement, BindError> {
        let name_part = stmt.name.0.into_iter().take(1).next().unwrap();
        let table_name = match name_part {
            sql::ObjectNamePart::Identifier(id) => id.value,
            x => return Err(BindError::UnsupportedTableName(x))
        };
        let primary_key = stmt.columns.iter()
            .map(|x| x.options.iter().find_map(|y| match y.option {
                sql::ColumnOption::PrimaryKey(_) => Some(x.name.value.clone()),
                _ => None
            }))
            .flatten()
            .collect::<Vec<String>>();
        let columns = stmt.columns.into_iter()
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
                // Take the first part of the qualified name.
                let name_part = name.0.into_iter().next().unwrap();
                match name_part {
                    sql::ObjectNamePart::Identifier(id) => id.value,
                    x => return Err(BindError::UnsupportedTableName(x)),
                }
            }
            None => String::new(),
        };

        // Extract the table name from the statement (e.g., "t" in CREATE INDEX idx ON t(col)).
        let table_name_part = stmt.table_name.0.into_iter().next().unwrap();
        let table_name = match table_name_part {
            sql::ObjectNamePart::Identifier(id) => id.value,
            x => return Err(BindError::UnsupportedTableName(x)),
        };

        // Look up the table in the catalog to obtain its OID and schema.
        let table_info = self.catalog.get_table_by_name(&table_name)
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

// bind select impl
impl<'cat> Binder<'cat> {

}
