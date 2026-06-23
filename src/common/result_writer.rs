use std::fmt::Display;

pub trait ResultWriter {
    fn add_row<T: Display>(&mut self, row: &[T]) -> &mut Self;
    fn add_header_row<T: Display>(&mut self, row: &[T]) -> &mut Self;
}

#[derive(Default)]
pub struct TableWriter {
    table: comfy_table::Table,
}

pub struct NoopWriter {}

impl TableWriter {
    /// Creates a new empty `TableWriter`.
    pub fn new() -> Self {
        TableWriter {
            table: comfy_table::Table::new(),
        }
    }

    /// Converts the table to its string representation.
    pub fn to_string(&self) -> String {
        self.table.to_string()
    }
}

impl ResultWriter for TableWriter {
    fn add_row<T: Display>(&mut self, row: &[T]) -> &mut Self {
        self.table.add_row(row);
        self
    }

    fn add_header_row<T: Display>(&mut self, row: &[T]) -> &mut Self {
        self.table.set_header(row);
        self
    }
}

impl ResultWriter for NoopWriter {
    fn add_row<T: Display>(&mut self, _row: &[T]) -> &mut Self {
        self
    }

    fn add_header_row<T: Display>(&mut self, _row: &[T]) -> &mut Self {
        self
    }
}
