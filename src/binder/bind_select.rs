use crate::binder::{BindError, Binder, ExpressionListRef, TableRef};
use sqlparser::ast as sql;

pub fn bind_values_list(binder: &Binder<'_>, sql_values: &sql::Values) -> Result<TableRef, BindError> {
    if sql_values.rows.is_empty() {
        return Err(BindError::Message("at least one row of values should be provided".to_string()));
    }
    let mut values = Vec::with_capacity(sql_values.rows.len());
    
    for row in &sql_values.rows {
        let mut row_values = Vec::with_capacity(row.content.len());
        for item in &row.content {
            row_values.push(binder.bind_expression(item)?);
        }
        values.push(row_values);
    }
    
    Ok(TableRef::ExpressionListRef(ExpressionListRef {
        values,
        identifier: "<unamed>".to_string(),
    }))
}
