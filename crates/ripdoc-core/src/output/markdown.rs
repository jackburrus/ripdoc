use crate::page::Page;
use crate::table::settings::TableSettings;

/// Extract page content as Markdown, including tables.
pub fn page_to_markdown(page: &Page, table_settings: &TableSettings) -> String {
    let mut result = String::new();

    // Detect tables
    let tables = crate::table::extract::extract_tables(page, table_settings);

    if tables.is_empty() {
        // No tables: just extract text
        let opts = crate::page::TextExtractOptions::default();
        return page.extract_text(&opts);
    }

    // Interleave text and tables based on vertical position
    let mut current_y = 0.0f64;

    // Sort tables by vertical position
    let mut sorted_tables = tables.clone();
    sorted_tables.sort_by(|a, b| a.bbox.top.partial_cmp(&b.bbox.top).unwrap());

    for table in &sorted_tables {
        // Extract text above this table
        let text_chars: Vec<_> = page
            .chars
            .iter()
            .filter(|c| c.top >= current_y && c.bottom <= table.bbox.top)
            .cloned()
            .collect();

        if !text_chars.is_empty() {
            let opts = crate::page::TextExtractOptions::default();
            let text =
                crate::text::extract::extract_text(&text_chars, page.width, page.height, &opts);
            if !text.trim().is_empty() {
                result.push_str(text.trim());
                result.push_str("\n\n");
            }
        }

        // Add table as markdown
        result.push_str(&table.to_markdown());
        result.push('\n');

        current_y = table.bbox.bottom;
    }

    // Extract text below last table
    let remaining_chars: Vec<_> = page
        .chars
        .iter()
        .filter(|c| c.top >= current_y)
        .cloned()
        .collect();

    if !remaining_chars.is_empty() {
        let opts = crate::page::TextExtractOptions::default();
        let text =
            crate::text::extract::extract_text(&remaining_chars, page.width, page.height, &opts);
        if !text.trim().is_empty() {
            result.push_str(text.trim());
            result.push('\n');
        }
    }

    result
}
