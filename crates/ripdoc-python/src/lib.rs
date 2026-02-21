mod page;
mod pdf;
mod table;

use pyo3::prelude::*;

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<pdf::PDF>()?;
    m.add_class::<page::PageWrapper>()?;
    m.add_class::<table::TableWrapper>()?;

    // Top-level open function (pdfplumber compatible)
    #[pyfunction]
    fn open(path: &str) -> PyResult<pdf::PDF> {
        pdf::PDF::open(path)
    }

    m.add_function(wrap_pyfunction!(open, m)?)?;

    // Version info
    m.add("__version__", "0.1.0")?;

    Ok(())
}
