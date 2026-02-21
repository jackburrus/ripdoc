use crate::error::Result;
use crate::geometry::BBox;
use crate::objects::*;

/// A page extracted from a PDF document.
/// Mirrors pdfplumber's Page interface.
#[derive(Debug, Clone)]
pub struct Page {
    pub page_number: usize,
    pub width: f64,
    pub height: f64,
    /// Cumulative offset from top of document.
    pub doctop_offset: f64,
    pub chars: Vec<Char>,
    pub lines: Vec<Line>,
    pub rects: Vec<Rect>,
    pub curves: Vec<Curve>,
}

impl Page {
    pub fn new(page_number: usize, width: f64, height: f64, doctop_offset: f64) -> Self {
        Self {
            page_number,
            width,
            height,
            doctop_offset,
            chars: Vec::new(),
            lines: Vec::new(),
            rects: Vec::new(),
            curves: Vec::new(),
        }
    }

    pub fn bbox(&self) -> BBox {
        BBox::new(0.0, 0.0, self.width, self.height)
    }

    /// Get all edges: explicit lines + rect edges.
    /// This is the key input for table detection.
    pub fn edges(&self) -> Vec<Line> {
        let mut edges = self.lines.clone();
        for rect in &self.rects {
            edges.extend(rect.to_edges());
        }
        edges
    }

    /// Get words by grouping characters.
    pub fn words(&self, x_tolerance: f64, y_tolerance: f64) -> Vec<Word> {
        crate::text::words::group_chars_to_words(&self.chars, x_tolerance, y_tolerance)
    }

    /// Extract text from the page.
    pub fn extract_text(&self, options: &TextExtractOptions) -> String {
        crate::text::extract::extract_text(&self.chars, self.width, self.height, options)
    }

    /// Crop the page to a bounding box, returning a new Page with only
    /// objects within the bbox.
    pub fn crop(&self, bbox: BBox) -> Page {
        let mut page = Page::new(self.page_number, bbox.width(), bbox.height(), self.doctop_offset);

        // Filter chars within bbox
        page.chars = self
            .chars
            .iter()
            .filter(|c| bbox.contains_point(c.x0, c.top) || bbox.contains_point(c.x1, c.bottom))
            .cloned()
            .collect();

        // Filter lines that intersect bbox
        page.lines = self
            .lines
            .iter()
            .filter(|l| bbox.intersects(&l.bbox()))
            .cloned()
            .collect();

        // Filter rects that intersect bbox
        page.rects = self
            .rects
            .iter()
            .filter(|r| bbox.intersects(&r.bbox()))
            .cloned()
            .collect();

        // Filter curves that intersect bbox
        page.curves = self
            .curves
            .iter()
            .filter(|c| bbox.intersects(&c.bbox()))
            .cloned()
            .collect();

        page
    }

    /// Like crop but only includes objects fully within the bbox.
    pub fn within_bbox(&self, bbox: BBox) -> Page {
        let mut page = Page::new(self.page_number, bbox.width(), bbox.height(), self.doctop_offset);

        page.chars = self
            .chars
            .iter()
            .filter(|c| bbox.contains_bbox(&c.bbox()))
            .cloned()
            .collect();

        page.lines = self
            .lines
            .iter()
            .filter(|l| bbox.contains_bbox(&l.bbox()))
            .cloned()
            .collect();

        page.rects = self
            .rects
            .iter()
            .filter(|r| bbox.contains_bbox(&r.bbox()))
            .cloned()
            .collect();

        page.curves = self
            .curves
            .iter()
            .filter(|c| bbox.contains_bbox(&c.bbox()))
            .cloned()
            .collect();

        page
    }

    /// Filter objects by a predicate function.
    pub fn filter_chars<F: Fn(&Char) -> bool>(&self, pred: F) -> Vec<&Char> {
        self.chars.iter().filter(|c| pred(c)).collect()
    }

    /// Search for text on the page.
    pub fn search(&self, pattern: &str, regex: bool) -> Result<Vec<TextMatch>> {
        crate::text::search::search_page(self, pattern, regex)
    }
}

/// Options for text extraction.
#[derive(Debug, Clone)]
pub struct TextExtractOptions {
    /// Preserve spatial layout using character grid.
    pub layout: bool,
    /// Horizontal tolerance for grouping characters into words.
    pub x_tolerance: f64,
    /// Vertical tolerance for grouping characters into lines.
    pub y_tolerance: f64,
    /// Characters per point (horizontal density for layout mode).
    pub x_density: f64,
    /// Characters per point (vertical density for layout mode).
    pub y_density: f64,
    /// Keep blank characters in output.
    pub keep_blank_chars: bool,
}

impl Default for TextExtractOptions {
    fn default() -> Self {
        Self {
            layout: false,
            x_tolerance: 3.0,
            y_tolerance: 3.0,
            x_density: 7.25,
            y_density: 13.0,
            keep_blank_chars: false,
        }
    }
}

/// A text search match result.
#[derive(Debug, Clone)]
pub struct TextMatch {
    pub text: String,
    pub bbox: BBox,
    pub page_number: usize,
    pub char_indices: Vec<usize>,
}
