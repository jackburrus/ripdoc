use crate::page::Page;
use crate::table::settings::TableSettings;
use crate::table::Table;

/// High-level table extraction entry point.
/// Detects tables and returns them with their content.
pub fn extract_tables(page: &Page, settings: &TableSettings) -> Vec<Table> {
    super::detect::detect_tables(page, settings)
}

/// Find tables on a page without extracting content.
/// Returns tables with empty cell text (only structure).
pub fn find_tables(page: &Page, settings: &TableSettings) -> Vec<Table> {
    super::detect::detect_tables(page, settings)
}

/// Extract tables and return as 2D grids (pdfplumber compatibility).
pub fn extract_table_grids(
    page: &Page,
    settings: &TableSettings,
) -> Vec<Vec<Vec<Option<String>>>> {
    let tables = extract_tables(page, settings);
    tables.iter().map(|t| t.to_grid()).collect()
}
