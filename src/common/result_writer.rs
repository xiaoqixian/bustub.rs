pub trait ResultWriter {
    fn add_row(&mut self, row: &[String]) -> &mut Self;
    fn add_header_row(&mut self, row: &[String]) -> &mut Self;
}

pub struct TableWriter {
    table: comfy_table::Table
}

impl ResultWriter for TableWriter {
    fn add_row(&mut self, row: &[String]) -> &mut Self {
        self.table.add_row(row);
        self
    }

    fn add_header_row(&mut self, row: &[String]) -> &mut Self {
        self.table.set_header(row);
        self
    }
}
