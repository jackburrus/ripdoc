use pyo3::prelude::*;

/// Python wrapper for a detected table.
#[pyclass(name = "Table")]
#[derive(Clone)]
pub struct TableWrapper {
    pub(crate) inner: ripdoc_core::Table,
}

impl TableWrapper {
    pub fn from_table(table: ripdoc_core::Table) -> Self {
        Self { inner: table }
    }
}

#[pymethods]
impl TableWrapper {
    /// Get table bounding box as (x0, top, x1, bottom).
    #[getter]
    fn bbox(&self) -> (f64, f64, f64, f64) {
        (
            self.inner.bbox.x0,
            self.inner.bbox.top,
            self.inner.bbox.x1,
            self.inner.bbox.bottom,
        )
    }

    /// Number of rows.
    #[getter]
    fn row_count(&self) -> usize {
        self.inner.row_count
    }

    /// Number of columns.
    #[getter]
    fn col_count(&self) -> usize {
        self.inner.col_count
    }

    /// Extract table data as 2D list.
    fn extract(&self) -> Vec<Vec<Option<String>>> {
        self.inner.to_grid()
    }

    /// Convert to markdown string.
    fn to_markdown(&self) -> String {
        self.inner.to_markdown()
    }

    /// Convert to CSV string.
    fn to_csv(&self) -> String {
        self.inner.to_csv()
    }

    /// Convert to HTML string.
    fn to_html(&self) -> String {
        self.inner.to_html()
    }

    fn __repr__(&self) -> String {
        format!(
            "<Table rows={} cols={}>",
            self.inner.row_count, self.inner.col_count
        )
    }

    fn __len__(&self) -> usize {
        self.inner.row_count
    }
}
