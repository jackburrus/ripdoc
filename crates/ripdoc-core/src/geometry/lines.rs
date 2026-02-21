use super::bbox::BBox;

/// An edge (line segment) used in table detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub x0: f64,
    pub top: f64,
    pub x1: f64,
    pub bottom: f64,
    pub width: f64,
    pub orientation: Orientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Edge {
    pub fn new(x0: f64, top: f64, x1: f64, bottom: f64, width: f64) -> Self {
        let orientation = if (top - bottom).abs() < (x1 - x0).abs() {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        };
        Self {
            x0,
            top,
            x1,
            bottom,
            width,
            orientation,
        }
    }

    pub fn horizontal(x0: f64, x1: f64, y: f64, width: f64) -> Self {
        Self {
            x0: x0.min(x1),
            top: y,
            x1: x0.max(x1),
            bottom: y,
            width,
            orientation: Orientation::Horizontal,
        }
    }

    pub fn vertical(x: f64, top: f64, bottom: f64, width: f64) -> Self {
        Self {
            x0: x,
            top: top.min(bottom),
            x1: x,
            bottom: top.max(bottom),
            width,
            orientation: Orientation::Vertical,
        }
    }

    pub fn length(&self) -> f64 {
        match self.orientation {
            Orientation::Horizontal => (self.x1 - self.x0).abs(),
            Orientation::Vertical => (self.bottom - self.top).abs(),
        }
    }

    pub fn bbox(&self) -> BBox {
        BBox::new(self.x0, self.top, self.x1, self.bottom)
    }

    pub fn is_horizontal(&self) -> bool {
        self.orientation == Orientation::Horizontal
    }

    pub fn is_vertical(&self) -> bool {
        self.orientation == Orientation::Vertical
    }
}

/// Snap edges that are within tolerance of each other.
/// For horizontal edges with similar y-values, average them.
/// For vertical edges with similar x-values, average them.
pub fn snap_edges(edges: &mut [Edge], x_tolerance: f64, y_tolerance: f64) {
    // Snap horizontal edges
    let mut h_edges: Vec<&mut Edge> = edges
        .iter_mut()
        .filter(|e| e.is_horizontal())
        .collect();
    h_edges.sort_by(|a, b| a.top.partial_cmp(&b.top).unwrap());

    let mut i = 0;
    while i < h_edges.len() {
        let base_y = h_edges[i].top;
        let mut j = i + 1;
        while j < h_edges.len() && (h_edges[j].top - base_y).abs() <= y_tolerance {
            h_edges[j].top = base_y;
            h_edges[j].bottom = base_y;
            j += 1;
        }
        i = j;
    }

    // Snap vertical edges
    let mut v_edges: Vec<&mut Edge> = edges
        .iter_mut()
        .filter(|e| e.is_vertical())
        .collect();
    v_edges.sort_by(|a, b| a.x0.partial_cmp(&b.x0).unwrap());

    let mut i = 0;
    while i < v_edges.len() {
        let base_x = v_edges[i].x0;
        let mut j = i + 1;
        while j < v_edges.len() && (v_edges[j].x0 - base_x).abs() <= x_tolerance {
            v_edges[j].x0 = base_x;
            v_edges[j].x1 = base_x;
            j += 1;
        }
        i = j;
    }
}

/// Merge overlapping or adjacent edges of the same orientation.
pub fn merge_edges(edges: &[Edge], tolerance: f64) -> Vec<Edge> {
    let mut horizontals: Vec<Edge> = edges.iter().filter(|e| e.is_horizontal()).copied().collect();
    let mut verticals: Vec<Edge> = edges.iter().filter(|e| e.is_vertical()).copied().collect();

    let merged_h = merge_collinear_edges(&mut horizontals, tolerance, true);
    let merged_v = merge_collinear_edges(&mut verticals, tolerance, false);

    let mut result = merged_h;
    result.extend(merged_v);
    result
}

fn merge_collinear_edges(edges: &mut [Edge], tolerance: f64, horizontal: bool) -> Vec<Edge> {
    if edges.is_empty() {
        return vec![];
    }

    if horizontal {
        edges.sort_by(|a, b| {
            a.top
                .partial_cmp(&b.top)
                .unwrap()
                .then(a.x0.partial_cmp(&b.x0).unwrap())
        });
    } else {
        edges.sort_by(|a, b| {
            a.x0
                .partial_cmp(&b.x0)
                .unwrap()
                .then(a.top.partial_cmp(&b.top).unwrap())
        });
    }

    let mut merged = vec![edges[0]];

    for edge in &edges[1..] {
        let last = merged.last_mut().unwrap();
        let same_line = if horizontal {
            (last.top - edge.top).abs() <= tolerance
        } else {
            (last.x0 - edge.x0).abs() <= tolerance
        };

        let overlaps = if horizontal {
            edge.x0 <= last.x1 + tolerance
        } else {
            edge.top <= last.bottom + tolerance
        };

        if same_line && overlaps {
            // Merge: extend the last edge
            if horizontal {
                last.x1 = last.x1.max(edge.x1);
            } else {
                last.bottom = last.bottom.max(edge.bottom);
            }
            last.width = last.width.max(edge.width);
        } else {
            merged.push(*edge);
        }
    }

    merged
}

/// Find intersection points between horizontal and vertical edges.
pub fn find_intersections(
    edges: &[Edge],
    x_tolerance: f64,
    y_tolerance: f64,
) -> Vec<(f64, f64)> {
    let horizontals: Vec<&Edge> = edges.iter().filter(|e| e.is_horizontal()).collect();
    let verticals: Vec<&Edge> = edges.iter().filter(|e| e.is_vertical()).collect();

    let mut intersections = Vec::new();

    for h in &horizontals {
        for v in &verticals {
            // Check if horizontal y is within vertical's y range
            let y_in_range = h.top >= v.top - y_tolerance && h.top <= v.bottom + y_tolerance;
            // Check if vertical x is within horizontal's x range
            let x_in_range = v.x0 >= h.x0 - x_tolerance && v.x0 <= h.x1 + x_tolerance;

            if y_in_range && x_in_range {
                intersections.push((v.x0, h.top));
            }
        }
    }

    // Deduplicate intersections within tolerance
    intersections.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(a.0.partial_cmp(&b.0).unwrap()));
    dedup_points(&mut intersections, x_tolerance, y_tolerance);

    intersections
}

fn dedup_points(points: &mut Vec<(f64, f64)>, x_tol: f64, y_tol: f64) {
    if points.len() <= 1 {
        return;
    }
    let mut write = 0;
    for read in 1..points.len() {
        if (points[read].0 - points[write].0).abs() > x_tol
            || (points[read].1 - points[write].1).abs() > y_tol
        {
            write += 1;
            points[write] = points[read];
        }
    }
    points.truncate(write + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let h = Edge::horizontal(10.0, 100.0, 50.0, 1.0);
        assert!(h.is_horizontal());
        assert_eq!(h.length(), 90.0);

        let v = Edge::vertical(50.0, 10.0, 100.0, 1.0);
        assert!(v.is_vertical());
        assert_eq!(v.length(), 90.0);
    }

    #[test]
    fn test_find_intersections() {
        let edges = vec![
            Edge::horizontal(0.0, 100.0, 50.0, 1.0),
            Edge::vertical(50.0, 0.0, 100.0, 1.0),
        ];
        let pts = find_intersections(&edges, 3.0, 3.0);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].0 - 50.0).abs() < 0.01);
        assert!((pts[0].1 - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_merge_edges() {
        let edges = vec![
            Edge::horizontal(0.0, 50.0, 10.0, 1.0),
            Edge::horizontal(48.0, 100.0, 10.0, 1.0),
        ];
        let merged = merge_edges(&edges, 3.0);
        assert_eq!(merged.len(), 1);
        assert!((merged[0].x0 - 0.0).abs() < 0.01);
        assert!((merged[0].x1 - 100.0).abs() < 0.01);
    }
}
