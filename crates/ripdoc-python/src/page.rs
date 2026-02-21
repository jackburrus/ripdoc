use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::table::TableWrapper;

/// Python wrapper for a PDF page.
#[pyclass(name = "Page")]
#[derive(Clone)]
pub struct PageWrapper {
    pub(crate) inner: ripdoc_core::Page,
}

impl PageWrapper {
    pub fn from_page(page: ripdoc_core::Page) -> Self {
        Self { inner: page }
    }
}

#[pymethods]
impl PageWrapper {
    /// Page number (1-indexed).
    #[getter]
    fn page_number(&self) -> usize {
        self.inner.page_number
    }

    /// Page width in points.
    #[getter]
    fn width(&self) -> f64 {
        self.inner.width
    }

    /// Page height in points.
    #[getter]
    fn height(&self) -> f64 {
        self.inner.height
    }

    /// Get all characters with positioning info.
    /// Returns list of dicts matching pdfplumber format.
    #[getter]
    fn chars(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();

        for ch in &self.inner.chars {
            let dict = PyDict::new(py);
            dict.set_item("text", &ch.text)?;
            dict.set_item("fontname", &ch.fontname)?;
            dict.set_item("size", ch.size)?;
            dict.set_item("x0", ch.x0)?;
            dict.set_item("x1", ch.x1)?;
            dict.set_item("top", ch.top)?;
            dict.set_item("bottom", ch.bottom)?;
            dict.set_item("doctop", ch.doctop)?;
            dict.set_item("upright", ch.upright)?;
            dict.set_item("adv", ch.adv)?;
            result.push(dict.into());
        }

        Ok(result)
    }

    /// Get all lines on the page.
    #[getter]
    fn lines(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();

        for line in &self.inner.lines {
            let dict = PyDict::new(py);
            dict.set_item("x0", line.x0)?;
            dict.set_item("y0", line.y0)?;
            dict.set_item("x1", line.x1)?;
            dict.set_item("y1", line.y1)?;
            dict.set_item("top", line.top)?;
            dict.set_item("bottom", line.bottom)?;
            dict.set_item("width", line.width)?;
            result.push(dict.into());
        }

        Ok(result)
    }

    /// Get all rectangles on the page.
    #[getter]
    fn rects(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();

        for rect in &self.inner.rects {
            let dict = PyDict::new(py);
            dict.set_item("x0", rect.x0)?;
            dict.set_item("top", rect.top)?;
            dict.set_item("x1", rect.x1)?;
            dict.set_item("bottom", rect.bottom)?;
            dict.set_item("width", rect.width)?;
            dict.set_item("height", rect.height)?;
            dict.set_item("linewidth", rect.linewidth)?;
            result.push(dict.into());
        }

        Ok(result)
    }

    /// Get all edges (lines + rect edges).
    #[getter]
    fn edges(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();

        for edge in self.inner.edges() {
            let dict = PyDict::new(py);
            dict.set_item("x0", edge.x0)?;
            dict.set_item("y0", edge.y0)?;
            dict.set_item("x1", edge.x1)?;
            dict.set_item("y1", edge.y1)?;
            dict.set_item("top", edge.top)?;
            dict.set_item("bottom", edge.bottom)?;
            dict.set_item("width", edge.width)?;
            result.push(dict.into());
        }

        Ok(result)
    }

    /// Extract text from the page.
    #[pyo3(signature = (**kwargs))]
    fn extract_text(&self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<String> {
        let mut opts = ripdoc_core::TextExtractOptions::default();

        if let Some(kw) = kwargs {
            if let Some(layout) = kw.get_item("layout")? {
                opts.layout = layout.extract::<bool>()?;
            }
            if let Some(xt) = kw.get_item("x_tolerance")? {
                opts.x_tolerance = xt.extract::<f64>()?;
            }
            if let Some(yt) = kw.get_item("y_tolerance")? {
                opts.y_tolerance = yt.extract::<f64>()?;
            }
            if let Some(xd) = kw.get_item("x_density")? {
                opts.x_density = xd.extract::<f64>()?;
            }
            if let Some(yd) = kw.get_item("y_density")? {
                opts.y_density = yd.extract::<f64>()?;
            }
        }

        Ok(self.inner.extract_text(&opts))
    }

    /// Extract words from the page.
    #[pyo3(signature = (x_tolerance=3.0, y_tolerance=3.0))]
    fn extract_words(
        &self,
        py: Python<'_>,
        x_tolerance: f64,
        y_tolerance: f64,
    ) -> PyResult<Vec<PyObject>> {
        let words = self.inner.words(x_tolerance, y_tolerance);
        let mut result = Vec::new();

        for word in &words {
            let dict = PyDict::new(py);
            dict.set_item("text", &word.text)?;
            dict.set_item("x0", word.x0)?;
            dict.set_item("x1", word.x1)?;
            dict.set_item("top", word.top)?;
            dict.set_item("bottom", word.bottom)?;
            dict.set_item("doctop", word.doctop)?;
            dict.set_item("upright", word.upright)?;
            result.push(dict.into());
        }

        Ok(result)
    }

    /// Find tables on the page.
    #[pyo3(signature = (table_settings=None))]
    fn find_tables(
        &self,
        table_settings: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Vec<TableWrapper>> {
        let settings = parse_table_settings(table_settings)?;
        let tables = ripdoc_core::table::extract::find_tables(&self.inner, &settings);

        Ok(tables.into_iter().map(TableWrapper::from_table).collect())
    }

    /// Extract tables as 2D grids.
    #[pyo3(signature = (table_settings=None))]
    fn extract_tables(
        &self,
        table_settings: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Vec<Vec<Vec<Option<String>>>>> {
        let settings = parse_table_settings(table_settings)?;
        Ok(ripdoc_core::table::extract::extract_table_grids(
            &self.inner,
            &settings,
        ))
    }

    /// Extract the first table as a 2D grid.
    #[pyo3(signature = (table_settings=None))]
    fn extract_table(
        &self,
        table_settings: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<Vec<Vec<Option<String>>>>> {
        let settings = parse_table_settings(table_settings)?;
        let grids = ripdoc_core::table::extract::extract_table_grids(&self.inner, &settings);
        Ok(grids.into_iter().next())
    }

    /// Crop the page to a bounding box.
    fn crop(&self, bbox: (f64, f64, f64, f64)) -> PyResult<PageWrapper> {
        let bb = ripdoc_core::BBox::new(bbox.0, bbox.1, bbox.2, bbox.3);
        Ok(PageWrapper::from_page(self.inner.crop(bb)))
    }

    /// Get objects within a bounding box (strict containment).
    fn within_bbox(&self, bbox: (f64, f64, f64, f64)) -> PyResult<PageWrapper> {
        let bb = ripdoc_core::BBox::new(bbox.0, bbox.1, bbox.2, bbox.3);
        Ok(PageWrapper::from_page(self.inner.within_bbox(bb)))
    }

    /// Search for text on the page.
    #[pyo3(signature = (pattern, regex=false))]
    fn search(
        &self,
        py: Python<'_>,
        pattern: &str,
        regex: bool,
    ) -> PyResult<Vec<PyObject>> {
        let matches = self.inner.search(pattern, regex).map_err(|e| {
            PyValueError::new_err(format!("Search error: {}", e))
        })?;

        let mut result = Vec::new();
        for m in &matches {
            let dict = PyDict::new(py);
            dict.set_item("text", &m.text)?;
            dict.set_item("x0", m.bbox.x0)?;
            dict.set_item("top", m.bbox.top)?;
            dict.set_item("x1", m.bbox.x1)?;
            dict.set_item("bottom", m.bbox.bottom)?;
            dict.set_item("page_number", m.page_number)?;
            result.push(dict.into());
        }

        Ok(result)
    }

    /// Get bounding box of the page.
    #[getter]
    fn bbox(&self) -> (f64, f64, f64, f64) {
        (0.0, 0.0, self.inner.width, self.inner.height)
    }

    fn __repr__(&self) -> String {
        format!(
            "<Page number={} width={:.1} height={:.1}>",
            self.inner.page_number, self.inner.width, self.inner.height
        )
    }
}

fn parse_table_settings(
    settings: Option<&Bound<'_, PyDict>>,
) -> PyResult<ripdoc_core::TableSettings> {
    let mut ts = ripdoc_core::TableSettings::default();

    if let Some(kw) = settings {
        if let Some(vs) = kw.get_item("vertical_strategy")? {
            ts.vertical_strategy = parse_strategy(vs.extract::<String>()?.as_str())?;
        }
        if let Some(hs) = kw.get_item("horizontal_strategy")? {
            ts.horizontal_strategy = parse_strategy(hs.extract::<String>()?.as_str())?;
        }
        if let Some(st) = kw.get_item("snap_tolerance")? {
            ts.snap_tolerance = st.extract::<f64>()?;
        }
        if let Some(jt) = kw.get_item("join_tolerance")? {
            ts.join_tolerance = jt.extract::<f64>()?;
        }
        if let Some(eml) = kw.get_item("edge_min_length")? {
            ts.edge_min_length = eml.extract::<f64>()?;
        }
        if let Some(mwv) = kw.get_item("min_words_vertical")? {
            ts.min_words_vertical = mwv.extract::<usize>()?;
        }
        if let Some(mwh) = kw.get_item("min_words_horizontal")? {
            ts.min_words_horizontal = mwh.extract::<usize>()?;
        }
        if let Some(it) = kw.get_item("intersection_tolerance")? {
            ts.intersection_tolerance = it.extract::<f64>()?;
        }
        if let Some(tt) = kw.get_item("text_tolerance")? {
            ts.text_tolerance = tt.extract::<f64>()?;
        }
        if let Some(evl) = kw.get_item("explicit_vertical_lines")? {
            ts.explicit_vertical_lines = evl.extract::<Vec<f64>>()?;
        }
        if let Some(ehl) = kw.get_item("explicit_horizontal_lines")? {
            ts.explicit_horizontal_lines = ehl.extract::<Vec<f64>>()?;
        }
    }

    Ok(ts)
}

fn parse_strategy(s: &str) -> PyResult<ripdoc_core::table::settings::Strategy> {
    match s {
        "lines" => Ok(ripdoc_core::table::settings::Strategy::Lines),
        "lines_strict" => Ok(ripdoc_core::table::settings::Strategy::LinesStrict),
        "text" => Ok(ripdoc_core::table::settings::Strategy::Text),
        "explicit" => Ok(ripdoc_core::table::settings::Strategy::Explicit),
        _ => Err(PyValueError::new_err(format!(
            "Unknown strategy: '{}'. Expected: lines, lines_strict, text, explicit",
            s
        ))),
    }
}
