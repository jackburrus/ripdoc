use crate::page::Page;
use crate::table::settings::TableSettings;

/// Convert page tables to HTML.
pub fn tables_to_html(page: &Page, table_settings: &TableSettings) -> String {
    let tables = crate::table::extract::extract_tables(page, table_settings);
    tables
        .iter()
        .map(|t| t.to_html())
        .collect::<Vec<_>>()
        .join("\n\n")
}
