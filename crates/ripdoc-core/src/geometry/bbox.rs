use serde::Serialize;

/// Bounding box in pdfplumber coordinate system (origin at top-left).
/// x0 < x1, top < bottom (top is closer to page top, so smaller value).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BBox {
    pub x0: f64,
    pub top: f64,
    pub x1: f64,
    pub bottom: f64,
}

impl BBox {
    pub fn new(x0: f64, top: f64, x1: f64, bottom: f64) -> Self {
        Self { x0, top, x1, bottom }
    }

    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }

    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    pub fn center_x(&self) -> f64 {
        (self.x0 + self.x1) / 2.0
    }

    pub fn center_y(&self) -> f64 {
        (self.top + self.bottom) / 2.0
    }

    /// Check if this bbox contains a point.
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.top && y <= self.bottom
    }

    /// Check if this bbox intersects another.
    pub fn intersects(&self, other: &BBox) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && self.top < other.bottom && self.bottom > other.top
    }

    /// Check if this bbox fully contains another.
    pub fn contains_bbox(&self, other: &BBox) -> bool {
        self.x0 <= other.x0 && self.x1 >= other.x1 && self.top <= other.top && self.bottom >= other.bottom
    }

    /// Compute the intersection of two bboxes.
    pub fn intersection(&self, other: &BBox) -> Option<BBox> {
        let x0 = self.x0.max(other.x0);
        let top = self.top.max(other.top);
        let x1 = self.x1.min(other.x1);
        let bottom = self.bottom.min(other.bottom);

        if x0 < x1 && top < bottom {
            Some(BBox::new(x0, top, x1, bottom))
        } else {
            None
        }
    }

    /// Compute the union (bounding box) of two bboxes.
    pub fn union(&self, other: &BBox) -> BBox {
        BBox::new(
            self.x0.min(other.x0),
            self.top.min(other.top),
            self.x1.max(other.x1),
            self.bottom.max(other.bottom),
        )
    }

    /// Expand the bbox by a margin on all sides.
    pub fn expand(&self, margin: f64) -> BBox {
        BBox::new(
            self.x0 - margin,
            self.top - margin,
            self.x1 + margin,
            self.bottom + margin,
        )
    }

    /// Convert from PDF coordinates (origin bottom-left) to pdfplumber coordinates (origin top-left).
    pub fn from_pdf_coords(x0: f64, y0: f64, x1: f64, y1: f64, page_height: f64) -> Self {
        let top = page_height - y1.max(y0);
        let bottom = page_height - y1.min(y0);
        BBox::new(x0.min(x1), top, x0.max(x1), bottom)
    }
}

impl Default for BBox {
    fn default() -> Self {
        Self {
            x0: 0.0,
            top: 0.0,
            x1: 0.0,
            bottom: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_basic() {
        let bbox = BBox::new(10.0, 20.0, 100.0, 80.0);
        assert_eq!(bbox.width(), 90.0);
        assert_eq!(bbox.height(), 60.0);
        assert!(bbox.contains_point(50.0, 50.0));
        assert!(!bbox.contains_point(5.0, 50.0));
    }

    #[test]
    fn test_bbox_intersection() {
        let a = BBox::new(0.0, 0.0, 100.0, 100.0);
        let b = BBox::new(50.0, 50.0, 150.0, 150.0);
        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter, BBox::new(50.0, 50.0, 100.0, 100.0));
    }

    #[test]
    fn test_pdf_coord_conversion() {
        let bbox = BBox::from_pdf_coords(72.0, 700.0, 200.0, 750.0, 792.0);
        assert!((bbox.top - 42.0).abs() < 0.01);
        assert!((bbox.bottom - 92.0).abs() < 0.01);
    }
}
