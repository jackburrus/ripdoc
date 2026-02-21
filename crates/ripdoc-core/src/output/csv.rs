use crate::page::Page;
use crate::table::settings::TableSettings;

/// Convert page tables to CSV.
pub fn tables_to_csv(page: &Page, table_settings: &TableSettings) -> Vec<String> {
    let tables = crate::table::extract::extract_tables(page, table_settings);
    tables.iter().map(|t| t.to_csv()).collect()
}
