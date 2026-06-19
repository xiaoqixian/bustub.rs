use std::fmt::Display;

pub trait ResultWriter {
    fn add_row<T: Display>(&mut self, row: &[T]) -> &mut Self;
    fn add_header_row<T: Display>(&mut self, row: &[T]) -> &mut Self;
}

pub struct TableWriter {
    table: comfy_table::Table,
}

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

impl Default for TableWriter {
    fn default() -> Self {
        Self::new()
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
