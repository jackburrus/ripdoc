use std::sync::Arc;

use serde::Serialize;

use crate::geometry::{BBox, Matrix};

/// A color value extracted from PDF.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Color {
    Gray(f64),
    RGB(f64, f64, f64),
    CMYK(f64, f64, f64, f64),
}

/// A single character extracted from a PDF page with full positioning info.
/// Matches pdfplumber's char dict format.
#[derive(Debug, Clone, Serialize)]
pub struct Char {
    pub text: String,
    pub fontname: String,
    pub size: f64,
    pub x0: f64,
    pub x1: f64,
    pub top: f64,
    pub bottom: f64,
    /// Distance from top of *document* (cumulative page heights).
    pub doctop: f64,
    /// The text rendering matrix [a, b, c, d, e, f].
    pub matrix: [f64; 6],
    /// Whether the text is upright (not rotated).
    pub upright: bool,
    pub stroking_color: Arc<Option<Color>>,
    pub non_stroking_color: Arc<Option<Color>>,
    /// Width of the character advance (for spacing calculations).
    pub adv: f64,
}

impl Char {
    pub fn bbox(&self) -> BBox {
        BBox::new(self.x0, self.top, self.x1, self.bottom)
    }
}

/// A line segment on the page.
#[derive(Debug, Clone, Serialize)]
pub struct Line {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub top: f64,
    pub bottom: f64,
    pub width: f64,
    pub stroking_color: Arc<Option<Color>>,
    pub non_stroking_color: Arc<Option<Color>>,
}

impl Line {
    pub fn bbox(&self) -> BBox {
        BBox::new(
            self.x0.min(self.x1),
            self.top,
            self.x0.max(self.x1),
            self.bottom,
        )
    }

    pub fn is_horizontal(&self) -> bool {
        (self.y0 - self.y1).abs() < 1e-6
    }

    pub fn is_vertical(&self) -> bool {
        (self.x0 - self.x1).abs() < 1e-6
    }

    pub fn length(&self) -> f64 {
        ((self.x1 - self.x0).powi(2) + (self.y1 - self.y0).powi(2)).sqrt()
    }
}

/// A rectangle on the page.
#[derive(Debug, Clone, Serialize)]
pub struct Rect {
    pub x0: f64,
    pub top: f64,
    pub x1: f64,
    pub bottom: f64,
    pub width: f64,
    pub height: f64,
    pub linewidth: f64,
    pub stroking_color: Arc<Option<Color>>,
    pub non_stroking_color: Arc<Option<Color>>,
}

impl Rect {
    pub fn bbox(&self) -> BBox {
        BBox::new(self.x0, self.top, self.x1, self.bottom)
    }

    /// Decompose rectangle into 4 edges (lines), matching pdfplumber's rect_edges.
    pub fn to_edges(&self) -> Vec<Line> {
        vec![
            // Top edge
            Line {
                x0: self.x0,
                y0: self.top,
                x1: self.x1,
                y1: self.top,
                top: self.top,
                bottom: self.top,
                width: self.linewidth,
                stroking_color: self.stroking_color.clone(),
                non_stroking_color: self.non_stroking_color.clone(),
            },
            // Bottom edge
            Line {
                x0: self.x0,
                y0: self.bottom,
                x1: self.x1,
                y1: self.bottom,
                top: self.bottom,
                bottom: self.bottom,
                width: self.linewidth,
                stroking_color: self.stroking_color.clone(),
                non_stroking_color: self.non_stroking_color.clone(),
            },
            // Left edge
            Line {
                x0: self.x0,
                y0: self.top,
                x1: self.x0,
                y1: self.bottom,
                top: self.top,
                bottom: self.bottom,
                width: self.linewidth,
                stroking_color: self.stroking_color.clone(),
                non_stroking_color: self.non_stroking_color.clone(),
            },
            // Right edge
            Line {
                x0: self.x1,
                y0: self.top,
                x1: self.x1,
                y1: self.bottom,
                top: self.top,
                bottom: self.bottom,
                width: self.linewidth,
                stroking_color: self.stroking_color.clone(),
                non_stroking_color: self.non_stroking_color.clone(),
            },
        ]
    }
}

/// A Bezier curve on the page.
#[derive(Debug, Clone, Serialize)]
pub struct Curve {
    pub points: Vec<(f64, f64)>,
    pub width: f64,
    pub stroking_color: Arc<Option<Color>>,
    pub non_stroking_color: Arc<Option<Color>>,
}

impl Curve {
    pub fn bbox(&self) -> BBox {
        if self.points.is_empty() {
            return BBox::default();
        }
        let mut x0 = f64::MAX;
        let mut top = f64::MAX;
        let mut x1 = f64::MIN;
        let mut bottom = f64::MIN;
        for &(x, y) in &self.points {
            x0 = x0.min(x);
            top = top.min(y);
            x1 = x1.max(x);
            bottom = bottom.max(y);
        }
        BBox::new(x0, top, x1, bottom)
    }
}

/// A word (group of characters).
#[derive(Debug, Clone, Serialize)]
pub struct Word {
    pub text: String,
    pub x0: f64,
    pub x1: f64,
    pub top: f64,
    pub bottom: f64,
    pub doctop: f64,
    pub upright: bool,
    pub fontname: String,
    pub size: f64,
}

impl Word {
    pub fn bbox(&self) -> BBox {
        BBox::new(self.x0, self.top, self.x1, self.bottom)
    }
}

/// Graphics state tracked during content stream interpretation.
#[derive(Debug, Clone)]
pub struct GraphicsState {
    pub ctm: Matrix,
    pub line_width: f64,
    pub line_cap: i32,
    pub line_join: i32,
    pub miter_limit: f64,
    pub dash_pattern: Vec<f64>,
    pub dash_phase: f64,
    pub stroking_color: Arc<Option<Color>>,
    pub non_stroking_color: Arc<Option<Color>>,
    pub stroking_colorspace: String,
    pub non_stroking_colorspace: String,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::identity(),
            line_width: 1.0,
            line_cap: 0,
            line_join: 0,
            miter_limit: 10.0,
            dash_pattern: vec![],
            dash_phase: 0.0,
            stroking_color: Arc::new(Some(Color::Gray(0.0))),
            non_stroking_color: Arc::new(Some(Color::Gray(0.0))),
            stroking_colorspace: "DeviceGray".into(),
            non_stroking_colorspace: "DeviceGray".into(),
        }
    }
}

/// Text state tracked during content stream interpretation.
#[derive(Debug, Clone)]
pub struct TextState {
    pub font_name: String,
    pub font_size: f64,
    pub char_spacing: f64,
    pub word_spacing: f64,
    pub horizontal_scaling: f64,
    pub leading: f64,
    pub rise: f64,
    pub render_mode: i32,
    /// Text matrix (Tm).
    pub text_matrix: Matrix,
    /// Text line matrix (saved at beginning of each line).
    pub text_line_matrix: Matrix,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            font_name: String::new(),
            font_size: 0.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            leading: 0.0,
            rise: 0.0,
            render_mode: 0,
            text_matrix: Matrix::identity(),
            text_line_matrix: Matrix::identity(),
        }
    }
}
