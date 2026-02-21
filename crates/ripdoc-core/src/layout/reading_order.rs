use crate::geometry::BBox;
use crate::objects::Char;

/// Determine reading order of text blocks using the XY-cut algorithm.
///
/// The XY-cut algorithm recursively splits the page into blocks:
/// 1. Find the largest horizontal gap → split into top/bottom
/// 2. Find the largest vertical gap → split into left/right
/// 3. Recurse until blocks are small enough
///
/// This handles multi-column layouts, headers, footers, etc.
pub fn determine_reading_order(chars: &[Char], page_bbox: &BBox) -> Vec<Vec<usize>> {
    if chars.is_empty() {
        return vec![];
    }

    let indices: Vec<usize> = (0..chars.len()).collect();
    let mut blocks = Vec::new();
    xy_cut(chars, &indices, page_bbox, &mut blocks, 20.0);

    if blocks.is_empty() {
        // If no cuts were made, return all chars as one block
        blocks.push(indices);
    }

    blocks
}

fn xy_cut(
    chars: &[Char],
    indices: &[usize],
    _bbox: &BBox,
    result: &mut Vec<Vec<usize>>,
    min_gap: f64,
) {
    if indices.is_empty() {
        return;
    }

    if indices.len() <= 3 {
        result.push(indices.to_vec());
        return;
    }

    // Try horizontal cut first (split into top/bottom)
    if let Some((top_indices, bottom_indices)) = find_horizontal_cut(chars, indices, min_gap) {
        let top_bbox = compute_bbox(chars, &top_indices);
        let bottom_bbox = compute_bbox(chars, &bottom_indices);
        xy_cut(chars, &top_indices, &top_bbox, result, min_gap);
        xy_cut(chars, &bottom_indices, &bottom_bbox, result, min_gap);
        return;
    }

    // Try vertical cut (split into left/right)
    if let Some((left_indices, right_indices)) = find_vertical_cut(chars, indices, min_gap) {
        let left_bbox = compute_bbox(chars, &left_indices);
        let right_bbox = compute_bbox(chars, &right_indices);
        xy_cut(chars, &left_indices, &left_bbox, result, min_gap);
        xy_cut(chars, &right_indices, &right_bbox, result, min_gap);
        return;
    }

    // No more cuts possible, this is a leaf block
    let mut sorted = indices.to_vec();
    sorted.sort_by(|&a, &b| {
        let ya = chars[a].top;
        let yb = chars[b].top;
        if (ya - yb).abs() <= 3.0 {
            chars[a].x0.partial_cmp(&chars[b].x0).unwrap()
        } else {
            ya.partial_cmp(&yb).unwrap()
        }
    });
    result.push(sorted);
}

fn find_horizontal_cut(
    chars: &[Char],
    indices: &[usize],
    min_gap: f64,
) -> Option<(Vec<usize>, Vec<usize>)> {
    // Sort by y position
    let mut sorted: Vec<usize> = indices.to_vec();
    sorted.sort_by(|&a, &b| chars[a].top.partial_cmp(&chars[b].top).unwrap());

    // Find the largest gap in y positions
    let mut best_gap = 0.0f64;
    let mut best_split = 0;

    for i in 0..sorted.len() - 1 {
        let gap = chars[sorted[i + 1]].top - chars[sorted[i]].bottom;
        if gap > best_gap {
            best_gap = gap;
            best_split = i + 1;
        }
    }

    if best_gap >= min_gap && best_split > 0 && best_split < sorted.len() {
        let top = sorted[..best_split].to_vec();
        let bottom = sorted[best_split..].to_vec();
        Some((top, bottom))
    } else {
        None
    }
}

fn find_vertical_cut(
    chars: &[Char],
    indices: &[usize],
    min_gap: f64,
) -> Option<(Vec<usize>, Vec<usize>)> {
    let mut sorted: Vec<usize> = indices.to_vec();
    sorted.sort_by(|&a, &b| chars[a].x0.partial_cmp(&chars[b].x0).unwrap());

    let mut best_gap = 0.0f64;
    let mut best_split = 0;

    for i in 0..sorted.len() - 1 {
        let gap = chars[sorted[i + 1]].x0 - chars[sorted[i]].x1;
        if gap > best_gap {
            best_gap = gap;
            best_split = i + 1;
        }
    }

    if best_gap >= min_gap && best_split > 0 && best_split < sorted.len() {
        let left = sorted[..best_split].to_vec();
        let right = sorted[best_split..].to_vec();
        Some((left, right))
    } else {
        None
    }
}

fn compute_bbox(chars: &[Char], indices: &[usize]) -> BBox {
    if indices.is_empty() {
        return BBox::default();
    }

    let mut x0 = f64::MAX;
    let mut top = f64::MAX;
    let mut x1 = f64::MIN;
    let mut bottom = f64::MIN;

    for &i in indices {
        x0 = x0.min(chars[i].x0);
        top = top.min(chars[i].top);
        x1 = x1.max(chars[i].x1);
        bottom = bottom.max(chars[i].bottom);
    }

    BBox::new(x0, top, x1, bottom)
}
