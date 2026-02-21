use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("PDF parsing error: {0}")]
    PdfParse(String),

    #[error("Font error: {0}")]
    Font(String),

    #[error("Content stream error: {0}")]
    ContentStream(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Table detection error: {0}")]
    TableDetection(String),

    #[error("Page {0} not found")]
    PageNotFound(usize),

    #[error("Invalid bbox: {0}")]
    InvalidBBox(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
