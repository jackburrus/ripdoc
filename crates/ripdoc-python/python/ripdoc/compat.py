"""
Drop-in replacement for pdfplumber.

Usage:
    import ripdoc.compat as pdfplumber

    with pdfplumber.open("file.pdf") as pdf:
        for page in pdf.pages:
            tables = page.extract_tables()
            text = page.extract_text()
"""

from ripdoc import PDF, Page, Table, open

__all__ = ["PDF", "Page", "Table", "open"]
