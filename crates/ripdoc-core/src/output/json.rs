use serde::Serialize;

use crate::page::Page;
use crate::table::settings::TableSettings;

#[derive(Serialize)]
pub struct PageJson {
    pub page_number: usize,
    pub width: f64,
    pub height: f64,
    pub text: String,
    pub tables: Vec<TableJson>,
    pub char_count: usize,
    pub line_count: usize,
    pub rect_count: usize,
}

#[derive(Serialize)]
pub struct TableJson {
    pub bbox: [f64; 4],
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<Option<String>>>,
}

/// Convert a page to structured JSON representation.
pub fn page_to_json(page: &Page, table_settings: &TableSettings) -> PageJson {
    let opts = crate::page::TextExtractOptions::default();
    let text = page.extract_text(&opts);

    let tables = crate::table::extract::extract_tables(page, table_settings);
    let table_jsons: Vec<TableJson> = tables
        .iter()
        .map(|t| TableJson {
            bbox: [t.bbox.x0, t.bbox.top, t.bbox.x1, t.bbox.bottom],
            rows: t.row_count,
            cols: t.col_count,
            data: t.to_grid(),
        })
        .collect();

    PageJson {
        page_number: page.page_number,
        width: page.width,
        height: page.height,
        text,
        tables: table_jsons,
        char_count: page.chars.len(),
        line_count: page.lines.len(),
        rect_count: page.rects.len(),
    }
}

/// Serialize a page to JSON string.
pub fn page_to_json_string(page: &Page, table_settings: &TableSettings) -> String {
    let json = page_to_json(page, table_settings);
    serde_json::to_string_pretty(&json).unwrap_or_default()
}
