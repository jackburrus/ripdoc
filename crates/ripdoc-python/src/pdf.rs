use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::page::PageWrapper;

/// Python wrapper for a PDF document.
#[pyclass(name = "PDF")]
pub struct PDF {
    pub(crate) inner: Arc<Mutex<ripdoc_core::Document>>,
    path: String,
}

#[pymethods]
impl PDF {
    /// Open a PDF file.
    #[staticmethod]
    pub fn open(path: &str) -> PyResult<Self> {
        let doc = ripdoc_core::Document::open(path)
            .map_err(|e| PyValueError::new_err(format!("Failed to open PDF: {}", e)))?;

        Ok(PDF {
            inner: Arc::new(Mutex::new(doc)),
            path: path.to_string(),
        })
    }

    /// Open a PDF from bytes.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let doc = ripdoc_core::Document::from_bytes(data)
            .map_err(|e| PyValueError::new_err(format!("Failed to load PDF: {}", e)))?;

        Ok(PDF {
            inner: Arc::new(Mutex::new(doc)),
            path: "<bytes>".to_string(),
        })
    }

    /// Get all pages.
    #[getter]
    fn pages(&self) -> PyResult<Vec<PageWrapper>> {
        let mut doc = self.inner.lock().unwrap();
        let count = doc.page_count();
        let mut pages = Vec::new();

        for i in 1..=count {
            let page = doc.page(i).map_err(|e| {
                PyValueError::new_err(format!("Failed to extract page {}: {}", i, e))
            })?;
            pages.push(PageWrapper::from_page(page.clone()));
        }

        Ok(pages)
    }

    /// Get a single page by 1-indexed number.
    fn page(&self, page_number: usize) -> PyResult<PageWrapper> {
        let mut doc = self.inner.lock().unwrap();
        let page = doc.page(page_number).map_err(|e| {
            PyValueError::new_err(format!("Failed to extract page {}: {}", page_number, e))
        })?;
        Ok(PageWrapper::from_page(page.clone()))
    }

    /// Get page count.
    #[getter]
    fn page_count(&self) -> usize {
        self.inner.lock().unwrap().page_count()
    }

    /// Get document metadata.
    #[getter]
    fn metadata(&self) -> HashMap<String, String> {
        self.inner.lock().unwrap().metadata()
    }

    /// Context manager support.
    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_val: Option<&Bound<'_, PyAny>>,
        _exc_tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        Ok(false)
    }

    fn __repr__(&self) -> String {
        let count = self.inner.lock().unwrap().page_count();
        format!("<PDF path='{}' pages={}>", self.path, count)
    }

    fn __len__(&self) -> usize {
        self.inner.lock().unwrap().page_count()
    }
}
