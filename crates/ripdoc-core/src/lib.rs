pub mod content_stream;
pub mod document;
pub mod error;
pub mod fonts;
pub mod geometry;
pub mod layout;
pub mod objects;
pub mod output;
pub mod page;
pub mod table;
pub mod text;

pub use document::Document;
pub use error::{Error, Result};
pub use geometry::BBox;
pub use objects::{Char, Color, Curve, Line, Rect, Word};
pub use page::{Page, TextExtractOptions, TextMatch};
pub use table::{Table, TableCell, TableSettings};
