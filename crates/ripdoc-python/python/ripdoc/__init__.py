"""
ripdoc - The Polars of PDF extraction.

100x faster drop-in replacement for pdfplumber.
Built in Rust with PyO3 bindings.

Usage:
    import ripdoc

    with ripdoc.open("document.pdf") as pdf:
        for page in pdf.pages:
            text = page.extract_text()
            tables = page.extract_tables()
"""

from ripdoc._core import PDF, open, __version__
from ripdoc._core import Page
from ripdoc._core import Table

__all__ = ["PDF", "Page", "Table", "open", "__version__"]
