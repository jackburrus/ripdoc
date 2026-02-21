use serde::{Deserialize, Serialize};

/// Strategy for detecting table edges on a given axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Strategy {
    /// Use explicit PDF lines and rect edges.
    Lines,
    /// Use only unbroken lines (no gaps allowed).
    LinesStrict,
    /// Infer boundaries from word alignment.
    Text,
    /// User provides exact coordinates.
    Explicit,
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Lines
    }
}

/// Configuration for table detection, matching pdfplumber's TableSettings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSettings {
    pub vertical_strategy: Strategy,
    pub horizontal_strategy: Strategy,

    /// How close lines must be to snap together (default 3.0).
    pub snap_tolerance: f64,
    pub snap_x_tolerance: Option<f64>,
    pub snap_y_tolerance: Option<f64>,

    /// How close endpoints must be to join lines (default 3.0).
    pub join_tolerance: f64,
    pub join_x_tolerance: Option<f64>,
    pub join_y_tolerance: Option<f64>,

    /// Minimum edge length to consider (default 3.0).
    pub edge_min_length: f64,

    /// For "text" strategy: minimum words to infer vertical/horizontal lines.
    pub min_words_vertical: usize,
    pub min_words_horizontal: usize,

    /// How close a crossing must be to count as an intersection (default 3.0).
    pub intersection_tolerance: f64,
    pub intersection_x_tolerance: Option<f64>,
    pub intersection_y_tolerance: Option<f64>,

    /// Character spacing for text extraction within cells.
    pub text_tolerance: f64,
    pub text_x_tolerance: Option<f64>,
    pub text_y_tolerance: Option<f64>,

    /// User-provided explicit lines (for Strategy::Explicit).
    pub explicit_vertical_lines: Vec<f64>,
    pub explicit_horizontal_lines: Vec<f64>,
}

impl Default for TableSettings {
    fn default() -> Self {
        Self {
            vertical_strategy: Strategy::Lines,
            horizontal_strategy: Strategy::Lines,
            snap_tolerance: 3.0,
            snap_x_tolerance: None,
            snap_y_tolerance: None,
            join_tolerance: 3.0,
            join_x_tolerance: None,
            join_y_tolerance: None,
            edge_min_length: 3.0,
            min_words_vertical: 3,
            min_words_horizontal: 1,
            intersection_tolerance: 3.0,
            intersection_x_tolerance: None,
            intersection_y_tolerance: None,
            text_tolerance: 3.0,
            text_x_tolerance: None,
            text_y_tolerance: None,
            explicit_vertical_lines: Vec::new(),
            explicit_horizontal_lines: Vec::new(),
        }
    }
}

impl TableSettings {
    pub fn snap_x(&self) -> f64 {
        self.snap_x_tolerance.unwrap_or(self.snap_tolerance)
    }

    pub fn snap_y(&self) -> f64 {
        self.snap_y_tolerance.unwrap_or(self.snap_tolerance)
    }

    pub fn join_x(&self) -> f64 {
        self.join_x_tolerance.unwrap_or(self.join_tolerance)
    }

    pub fn join_y(&self) -> f64 {
        self.join_y_tolerance.unwrap_or(self.join_tolerance)
    }

    pub fn intersection_x(&self) -> f64 {
        self.intersection_x_tolerance
            .unwrap_or(self.intersection_tolerance)
    }

    pub fn intersection_y(&self) -> f64 {
        self.intersection_y_tolerance
            .unwrap_or(self.intersection_tolerance)
    }

    pub fn text_x(&self) -> f64 {
        self.text_x_tolerance.unwrap_or(self.text_tolerance)
    }

    pub fn text_y(&self) -> f64 {
        self.text_y_tolerance.unwrap_or(self.text_tolerance)
    }
}
