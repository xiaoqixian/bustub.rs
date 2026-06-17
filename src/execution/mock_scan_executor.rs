use crate::catalog::Schema;

static TA_LIST_2022: [&str; 11] = [
    "amstqq",      "durovo",     "joyceliaoo", "karthik-ramanathan-3006",
    "kush789",     "lmwnshn",    "mkpjnx",     "skyzh",
    "thepinetree", "timlee0119", "yliang412"
];

pub fn get_mock_table_schema_of(table: &str) -> Result<Schema, String> {
    match table {
        _ => Err(format!("mock table {} not found", table))
    }
}
