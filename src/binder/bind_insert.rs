use sqlparser::ast::{self as sql, TableObject};

use crate::binder::{BindError, Binder, InsertStatement};

pub fn bind_insert(binder: &mut Binder<'_>, insert: &sql::Insert) -> Result<InsertStatement, BindError> {
    let table_name = match &insert.table {
        TableObject::TableName(name) => extract_object_name(name)?,
        _ => return Err(BindError::UnsupportedJoinType(format!("{}", insert.table)))
    };
    
    let source = insert.source.as_ref().ok_or_else(|| BindError::Message("insert should provide values".to_string()))?;
    let select = binder.bind_query(source.as_ref())?;
    Ok(InsertStatement {
        table: binder.bind_base_table_ref(table_name, None)?,
        select,
    })
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
