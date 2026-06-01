use crate::catalog::Column;

pub struct CreateStatement {
    pub table: String,
    pub columns: Vec<Column>,
    pub primary_key: Vec<String>
}

pub enum SqlStatement {
    Create(CreateStatement)
}

