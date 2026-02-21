pub mod detect;
pub mod extract;
pub mod merge;
pub mod settings;

use serde::Serialize;

use crate::geometry::BBox;

pub use settings::TableSettings;

/// A detected table on a page.
#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub bbox: BBox,
    pub cells: Vec<TableCell>,
    pub row_count: usize,
    pub col_count: usize,
}

impl Table {
    /// Extract table data as a 2D grid of optional strings.
    /// This matches pdfplumber's `extract_table()` return format.
    pub fn to_grid(&self) -> Vec<Vec<Option<String>>> {
        let mut grid: Vec<Vec<Option<String>>> = vec![vec![None; self.col_count]; self.row_count];

        for cell in &self.cells {
            if cell.row < self.row_count && cell.col < self.col_count {
                let text = if cell.text.is_empty() {
                    None
                } else {
                    Some(cell.text.clone())
                };

                // For merged cells, fill in the spanned positions with the same value
                for r in cell.row..(cell.row + cell.row_span).min(self.row_count) {
                    for c in cell.col..(cell.col + cell.col_span).min(self.col_count) {
                        if r != cell.row || c != cell.col {
                            grid[r][c] = text.clone();
                        }
                    }
                }

                grid[cell.row][cell.col] = text;
            }
        }

        grid
    }

    /// Convert table to markdown format.
    pub fn to_markdown(&self) -> String {
        let grid = self.to_grid();
        if grid.is_empty() {
            return String::new();
        }

        let mut result = String::new();

        for (i, row) in grid.iter().enumerate() {
            let cells: Vec<String> = row
                .iter()
                .map(|c| c.as_deref().unwrap_or("").to_string())
                .collect();
            result.push_str("| ");
            result.push_str(&cells.join(" | "));
            result.push_str(" |\n");

            // Add header separator after first row
            if i == 0 {
                let sep: Vec<String> = row.iter().map(|_| "---".to_string()).collect();
                result.push_str("| ");
                result.push_str(&sep.join(" | "));
                result.push_str(" |\n");
            }
        }

        result
    }

    /// Convert table to CSV format.
    pub fn to_csv(&self) -> String {
        let grid = self.to_grid();
        let mut result = String::new();

        for row in &grid {
            let cells: Vec<String> = row
                .iter()
                .map(|c| {
                    let text = c.as_deref().unwrap_or("");
                    if text.contains(',') || text.contains('"') || text.contains('\n') {
                        format!("\"{}\"", text.replace('"', "\"\""))
                    } else {
                        text.to_string()
                    }
                })
                .collect();
            result.push_str(&cells.join(","));
            result.push('\n');
        }

        result
    }

    /// Convert table to HTML format.
    pub fn to_html(&self) -> String {
        let grid = self.to_grid();
        let mut result = String::from("<table>\n");

        for (i, row) in grid.iter().enumerate() {
            result.push_str("  <tr>\n");
            let tag = if i == 0 { "th" } else { "td" };
            for cell in row {
                let text = cell.as_deref().unwrap_or("");
                result.push_str(&format!("    <{}>{}</{}>\n", tag, text, tag));
            }
            result.push_str("  </tr>\n");
        }

        result.push_str("</table>");
        result
    }
}

/// A single cell in a detected table.
#[derive(Debug, Clone, Serialize)]
pub struct TableCell {
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub text: String,
    pub bbox: BBox,
}
