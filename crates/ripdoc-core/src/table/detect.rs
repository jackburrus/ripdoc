use crate::geometry::bbox::BBox;
use crate::geometry::lines::{self, Edge};
use crate::objects::{Char, Word};
use crate::page::Page;
use crate::table::settings::{Strategy, TableSettings};
use crate::table::{Table, TableCell};

/// Detect tables on a page using the Nurminen/Tabula algorithm.
///
/// Algorithm:
/// 1. Collect edges (lines + rect edges + text-inferred edges)
/// 2. Filter by minimum length
/// 3. Snap nearby edges together
/// 4. Merge overlapping edges
/// 5. Find intersections between horizontal and vertical edges
/// 6. Build rectangular cells from intersection points
/// 7. Group cells into table regions
pub fn detect_tables(page: &Page, settings: &TableSettings) -> Vec<Table> {
    // Step 1: Collect edges
    let mut edges = collect_edges(page, settings);

    if edges.is_empty() {
        return vec![];
    }

    // Step 2: Filter by minimum length
    edges.retain(|e| e.length() >= settings.edge_min_length);

    if edges.is_empty() {
        return vec![];
    }

    // Step 3: Snap nearby edges
    lines::snap_edges(&mut edges, settings.snap_x(), settings.snap_y());

    // Step 4: Merge overlapping edges
    let edges = lines::merge_edges(&edges, settings.join_x().max(settings.join_y()));

    // Step 5: Find intersections
    let intersections = lines::find_intersections(
        &edges,
        settings.intersection_x(),
        settings.intersection_y(),
    );

    if intersections.len() < 4 {
        // Need at least 4 intersections to form a cell
        return vec![];
    }

    // Step 6: Build cells from intersections
    let cells = build_cells(&intersections, &edges, settings);

    if cells.is_empty() {
        return vec![];
    }

    // Step 7: Group cells into tables
    group_cells_into_tables(cells, page, settings)
}

/// Collect edges from the page based on the configured strategies.
fn collect_edges(page: &Page, settings: &TableSettings) -> Vec<Edge> {
    let mut edges = Vec::new();

    // Collect vertical edges
    match &settings.vertical_strategy {
        Strategy::Lines | Strategy::LinesStrict => {
            for line in &page.lines {
                if line.is_vertical() {
                    edges.push(Edge::vertical(
                        line.x0,
                        line.top,
                        line.bottom,
                        line.width,
                    ));
                }
            }
            // Rect edges (vertical sides)
            for rect in &page.rects {
                edges.push(Edge::vertical(rect.x0, rect.top, rect.bottom, rect.linewidth));
                edges.push(Edge::vertical(rect.x1, rect.top, rect.bottom, rect.linewidth));
            }
        }
        Strategy::Text => {
            let words = page.words(settings.text_x(), settings.text_y());
            let text_edges = infer_vertical_edges_from_text(&words, settings);
            edges.extend(text_edges);
        }
        Strategy::Explicit => {
            let y_min = 0.0;
            let y_max = page.height;
            for &x in &settings.explicit_vertical_lines {
                edges.push(Edge::vertical(x, y_min, y_max, 1.0));
            }
        }
    }

    // Collect horizontal edges
    match &settings.horizontal_strategy {
        Strategy::Lines | Strategy::LinesStrict => {
            for line in &page.lines {
                if line.is_horizontal() {
                    edges.push(Edge::horizontal(
                        line.x0,
                        line.x1,
                        line.top,
                        line.width,
                    ));
                }
            }
            // Rect edges (horizontal sides)
            for rect in &page.rects {
                edges.push(Edge::horizontal(rect.x0, rect.x1, rect.top, rect.linewidth));
                edges.push(Edge::horizontal(rect.x0, rect.x1, rect.bottom, rect.linewidth));
            }
        }
        Strategy::Text => {
            let words = page.words(settings.text_x(), settings.text_y());
            let text_edges = infer_horizontal_edges_from_text(&words, settings);
            edges.extend(text_edges);
        }
        Strategy::Explicit => {
            let x_min = 0.0;
            let x_max = page.width;
            for &y in &settings.explicit_horizontal_lines {
                edges.push(Edge::horizontal(x_min, x_max, y, 1.0));
            }
        }
    }

    edges
}

/// Infer vertical table edges from word positions.
/// Words aligned in columns suggest vertical boundaries.
fn infer_vertical_edges_from_text(words: &[Word], settings: &TableSettings) -> Vec<Edge> {
    if words.is_empty() {
        return vec![];
    }

    let mut edges = Vec::new();

    // Cluster x0 positions of words to find column boundaries
    let mut x_positions: Vec<f64> = words.iter().map(|w| w.x0).collect();
    x_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let clusters = crate::geometry::clustering::cluster_values(&x_positions, settings.text_x());

    for cluster in &clusters {
        if cluster.len() >= settings.min_words_vertical {
            let x = cluster.iter().map(|&i| x_positions[i]).sum::<f64>() / cluster.len() as f64;
            let y_min = words.iter().map(|w| w.top).fold(f64::MAX, f64::min);
            let y_max = words.iter().map(|w| w.bottom).fold(f64::MIN, f64::max);
            edges.push(Edge::vertical(x, y_min, y_max, 0.5));
        }
    }

    // Also add right edges of rightmost words in each column
    let mut x1_positions: Vec<f64> = words.iter().map(|w| w.x1).collect();
    x1_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let x1_clusters = crate::geometry::clustering::cluster_values(&x1_positions, settings.text_x());
    for cluster in &x1_clusters {
        if cluster.len() >= settings.min_words_vertical {
            let x = cluster.iter().map(|&i| x1_positions[i]).sum::<f64>() / cluster.len() as f64;
            let y_min = words.iter().map(|w| w.top).fold(f64::MAX, f64::min);
            let y_max = words.iter().map(|w| w.bottom).fold(f64::MIN, f64::max);
            edges.push(Edge::vertical(x, y_min, y_max, 0.5));
        }
    }

    edges
}

/// Infer horizontal table edges from word positions.
fn infer_horizontal_edges_from_text(words: &[Word], settings: &TableSettings) -> Vec<Edge> {
    if words.is_empty() {
        return vec![];
    }

    let mut edges = Vec::new();

    // Cluster y positions (top of words = row top boundaries)
    let mut y_positions: Vec<f64> = words.iter().map(|w| w.top).collect();
    y_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let clusters = crate::geometry::clustering::cluster_values(&y_positions, settings.text_y());

    let x_min = words.iter().map(|w| w.x0).fold(f64::MAX, f64::min);
    let x_max = words.iter().map(|w| w.x1).fold(f64::MIN, f64::max);

    for cluster in &clusters {
        if cluster.len() >= settings.min_words_horizontal {
            let y = cluster.iter().map(|&i| y_positions[i]).sum::<f64>() / cluster.len() as f64;
            edges.push(Edge::horizontal(x_min, x_max, y, 0.5));
        }
    }

    // Also add bottom edges
    let mut y_bottom_positions: Vec<f64> = words.iter().map(|w| w.bottom).collect();
    y_bottom_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let bottom_clusters =
        crate::geometry::clustering::cluster_values(&y_bottom_positions, settings.text_y());

    for cluster in &bottom_clusters {
        if cluster.len() >= settings.min_words_horizontal {
            let y = cluster
                .iter()
                .map(|&i| y_bottom_positions[i])
                .sum::<f64>()
                / cluster.len() as f64;
            edges.push(Edge::horizontal(x_min, x_max, y, 0.5));
        }
    }

    edges
}

/// Build rectangular cells from intersection points.
fn build_cells(
    intersections: &[(f64, f64)],
    edges: &[Edge],
    settings: &TableSettings,
) -> Vec<CellRect> {
    let mut cells = Vec::new();

    // Get sorted unique x and y coordinates
    let mut xs: Vec<f64> = intersections.iter().map(|p| p.0).collect();
    let mut ys: Vec<f64> = intersections.iter().map(|p| p.1).collect();

    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs.dedup_by(|a, b| (*a - *b).abs() < settings.intersection_x());

    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < settings.intersection_y());

    // For each pair of adjacent x and y coordinates, check if a cell exists
    for i in 0..ys.len() - 1 {
        for j in 0..xs.len() - 1 {
            let x0 = xs[j];
            let x1 = xs[j + 1];
            let y0 = ys[i];
            let y1 = ys[i + 1];

            // Verify all four corners are actual intersection points
            let tol_x = settings.intersection_x();
            let tol_y = settings.intersection_y();

            let has_tl = has_point(intersections, x0, y0, tol_x, tol_y);
            let has_tr = has_point(intersections, x1, y0, tol_x, tol_y);
            let has_bl = has_point(intersections, x0, y1, tol_x, tol_y);
            let has_br = has_point(intersections, x1, y1, tol_x, tol_y);

            if has_tl && has_tr && has_bl && has_br {
                // Verify edges exist connecting the corners
                let has_top = has_edge_between(edges, x0, x1, y0, true, tol_x, tol_y);
                let has_bottom = has_edge_between(edges, x0, x1, y1, true, tol_x, tol_y);
                let has_left = has_edge_between(edges, y0, y1, x0, false, tol_x, tol_y);
                let has_right = has_edge_between(edges, y0, y1, x1, false, tol_x, tol_y);

                // Require at least top+bottom or left+right edges
                if (has_top && has_bottom) || (has_left && has_right) {
                    cells.push(CellRect {
                        bbox: BBox::new(x0, y0, x1, y1),
                        row_idx: i,
                        col_idx: j,
                    });
                }
            }
        }
    }

    cells
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CellRect {
    bbox: BBox,
    row_idx: usize,
    col_idx: usize,
}

fn has_point(points: &[(f64, f64)], x: f64, y: f64, tol_x: f64, tol_y: f64) -> bool {
    points
        .iter()
        .any(|(px, py)| (px - x).abs() <= tol_x && (py - y).abs() <= tol_y)
}

fn has_edge_between(
    edges: &[Edge],
    start: f64,
    end: f64,
    fixed: f64,
    horizontal: bool,
    tol_x: f64,
    tol_y: f64,
) -> bool {
    edges.iter().any(|e| {
        if horizontal {
            e.is_horizontal()
                && (e.top - fixed).abs() <= tol_y
                && e.x0 <= start + tol_x
                && e.x1 >= end - tol_x
        } else {
            e.is_vertical()
                && (e.x0 - fixed).abs() <= tol_x
                && e.top <= start + tol_y
                && e.bottom >= end - tol_y
        }
    })
}

/// Group cells into contiguous table regions.
fn group_cells_into_tables(
    cells: Vec<CellRect>,
    page: &Page,
    settings: &TableSettings,
) -> Vec<Table> {
    if cells.is_empty() {
        return vec![];
    }

    // Find contiguous groups of cells
    let mut groups = find_contiguous_groups(&cells);

    let mut tables = Vec::new();

    for group in &mut groups {
        if group.is_empty() {
            continue;
        }

        // Calculate table bounds
        let table_bbox = group.iter().fold(group[0].bbox, |acc, cell| acc.union(&cell.bbox));

        // Renumber rows and columns within this table
        let mut row_ys: Vec<f64> = group.iter().map(|c| c.bbox.top).collect();
        row_ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        row_ys.dedup_by(|a, b| (*a - *b).abs() < settings.intersection_y());

        let mut col_xs: Vec<f64> = group.iter().map(|c| c.bbox.x0).collect();
        col_xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        col_xs.dedup_by(|a, b| (*a - *b).abs() < settings.intersection_x());

        let row_count = row_ys.len();
        let col_count = col_xs.len();

        // Build table cells with text content
        let mut table_cells = Vec::new();

        for cell_rect in group.iter() {
            // Find row and column index
            let row = row_ys
                .iter()
                .position(|&y| (y - cell_rect.bbox.top).abs() < settings.intersection_y())
                .unwrap_or(0);
            let col = col_xs
                .iter()
                .position(|&x| (x - cell_rect.bbox.x0).abs() < settings.intersection_x())
                .unwrap_or(0);

            // Extract text within cell bbox
            let text = extract_cell_text(page, &cell_rect.bbox, settings);

            table_cells.push(TableCell {
                row,
                col,
                row_span: 1,
                col_span: 1,
                text,
                bbox: cell_rect.bbox,
            });
        }

        tables.push(Table {
            bbox: table_bbox,
            cells: table_cells,
            row_count,
            col_count,
        });
    }

    tables
}

/// Find groups of contiguous cells (cells that share edges).
fn find_contiguous_groups(cells: &[CellRect]) -> Vec<Vec<CellRect>> {
    if cells.is_empty() {
        return vec![];
    }

    let n = cells.len();
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], i: usize) -> usize {
        if parent[i] != i {
            parent[i] = find(parent, parent[i]);
        }
        parent[i]
    }

    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    // Union cells that are adjacent (share an edge)
    for i in 0..n {
        for j in i + 1..n {
            if cells_adjacent(&cells[i], &cells[j]) {
                union(&mut parent, i, j);
            }
        }
    }

    // Group by root
    let mut groups: std::collections::HashMap<usize, Vec<CellRect>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(cells[i].clone());
    }

    groups.into_values().collect()
}

fn cells_adjacent(a: &CellRect, b: &CellRect) -> bool {
    let tol = 1.0;

    // Share top or bottom edge
    let share_horizontal = ((a.bbox.top - b.bbox.top).abs() < tol
        || (a.bbox.bottom - b.bbox.bottom).abs() < tol
        || (a.bbox.top - b.bbox.bottom).abs() < tol
        || (a.bbox.bottom - b.bbox.top).abs() < tol)
        && a.bbox.x0 < b.bbox.x1 + tol
        && a.bbox.x1 > b.bbox.x0 - tol;

    // Share left or right edge
    let share_vertical = ((a.bbox.x0 - b.bbox.x0).abs() < tol
        || (a.bbox.x1 - b.bbox.x1).abs() < tol
        || (a.bbox.x0 - b.bbox.x1).abs() < tol
        || (a.bbox.x1 - b.bbox.x0).abs() < tol)
        && a.bbox.top < b.bbox.bottom + tol
        && a.bbox.bottom > b.bbox.top - tol;

    share_horizontal || share_vertical
}

/// Extract text content from characters within a cell bbox.
fn extract_cell_text(page: &Page, cell_bbox: &BBox, settings: &TableSettings) -> String {
    let chars: Vec<&Char> = page
        .chars
        .iter()
        .filter(|c| {
            let cx = (c.x0 + c.x1) / 2.0;
            let cy = (c.top + c.bottom) / 2.0;
            cell_bbox.contains_point(cx, cy)
        })
        .collect();

    if chars.is_empty() {
        return String::new();
    }

    // Sort by position
    let mut sorted = chars;
    sorted.sort_by(|a, b| {
        let y_cmp = a.top.partial_cmp(&b.top).unwrap();
        if (a.top - b.top).abs() <= settings.text_y() {
            a.x0.partial_cmp(&b.x0).unwrap()
        } else {
            y_cmp
        }
    });

    let mut text = String::new();
    let mut last_top = sorted[0].top;

    for ch in &sorted {
        if (ch.top - last_top).abs() > settings.text_y() {
            text.push('\n');
            last_top = ch.top;
        }
        text.push_str(&ch.text);
    }

    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_point() {
        let points = vec![(10.0, 20.0), (100.0, 20.0), (10.0, 50.0), (100.0, 50.0)];
        assert!(has_point(&points, 10.0, 20.0, 1.0, 1.0));
        assert!(!has_point(&points, 50.0, 35.0, 1.0, 1.0));
    }

    #[test]
    fn test_cells_adjacent() {
        let a = CellRect {
            bbox: BBox::new(0.0, 0.0, 50.0, 30.0),
            row_idx: 0,
            col_idx: 0,
        };
        let b = CellRect {
            bbox: BBox::new(50.0, 0.0, 100.0, 30.0),
            row_idx: 0,
            col_idx: 1,
        };
        assert!(cells_adjacent(&a, &b));
    }
}
