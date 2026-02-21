use crate::geometry::BBox;
use crate::geometry::lines::Edge;
use crate::table::Table;

/// Detect merged cells in a table by analyzing gaps in the grid lines.
///
/// A cell is considered merged if:
/// 1. An expected internal edge is missing (line gap detection)
/// 2. Text spans across where a column boundary would be
pub fn detect_merged_cells(table: &mut Table, edges: &[Edge], tolerance: f64) {
    if table.cells.is_empty() || table.row_count <= 1 || table.col_count <= 1 {
        return;
    }

    // Detect horizontal merges
    detect_horizontal_merges(table, edges, tolerance);

    // Detect vertical merges
    detect_vertical_merges(table, edges, tolerance);
}

fn detect_horizontal_merges(table: &mut Table, edges: &[Edge], tolerance: f64) {
    // Collect merge operations first, then apply them
    let mut merges: Vec<(usize, usize, BBox, String)> = Vec::new(); // (row, col, right_bbox, right_text)

    for row in 0..table.row_count {
        for col in 0..table.col_count - 1 {
            let left_cell = table.cells.iter().find(|c| c.row == row && c.col == col);
            let right_cell = table.cells.iter().find(|c| c.row == row && c.col == col + 1);

            if let (Some(left), Some(right)) = (left_cell, right_cell) {
                let boundary_x = left.bbox.x1;
                let y_top = left.bbox.top;
                let y_bottom = left.bbox.bottom;

                let has_edge = edges.iter().any(|e| {
                    e.is_vertical()
                        && (e.x0 - boundary_x).abs() <= tolerance
                        && e.top <= y_top + tolerance
                        && e.bottom >= y_bottom - tolerance
                });

                if !has_edge {
                    merges.push((row, col, right.bbox, right.text.clone()));
                }
            }
        }
    }

    // Apply merges
    for (row, col, right_bbox, right_text) in merges {
        if let Some(cell) = table.cells.iter_mut().find(|c| c.row == row && c.col == col) {
            cell.col_span += 1;
            cell.bbox = cell.bbox.union(&right_bbox);
            if !right_text.is_empty() {
                if !cell.text.is_empty() {
                    cell.text.push(' ');
                }
                cell.text.push_str(&right_text);
            }
        }
    }
}

fn detect_vertical_merges(table: &mut Table, edges: &[Edge], tolerance: f64) {
    let mut merges: Vec<(usize, usize, BBox, String)> = Vec::new();

    for col in 0..table.col_count {
        for row in 0..table.row_count - 1 {
            let top_cell = table.cells.iter().find(|c| c.row == row && c.col == col);
            let bottom_cell = table.cells.iter().find(|c| c.row == row + 1 && c.col == col);

            if let (Some(top), Some(bottom)) = (top_cell, bottom_cell) {
                let boundary_y = top.bbox.bottom;
                let x_left = top.bbox.x0;
                let x_right = top.bbox.x1;

                let has_edge = edges.iter().any(|e| {
                    e.is_horizontal()
                        && (e.top - boundary_y).abs() <= tolerance
                        && e.x0 <= x_left + tolerance
                        && e.x1 >= x_right - tolerance
                });

                if !has_edge {
                    merges.push((row, col, bottom.bbox, bottom.text.clone()));
                }
            }
        }
    }

    for (row, col, bottom_bbox, bottom_text) in merges {
        if let Some(cell) = table.cells.iter_mut().find(|c| c.row == row && c.col == col) {
            cell.row_span += 1;
            cell.bbox = cell.bbox.union(&bottom_bbox);
            if !bottom_text.is_empty() {
                if !cell.text.is_empty() {
                    cell.text.push('\n');
                }
                cell.text.push_str(&bottom_text);
            }
        }
    }
}
