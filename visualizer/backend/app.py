"""
Ripdoc Visualizer — FastAPI backend.

Thin API layer over the ripdoc Python package.
Upload a PDF, then query per-page chars/words/lines/rects/edges/tables/text/search.
"""

import tempfile
import os
import time
import statistics
from pathlib import Path
from typing import Optional, Callable, Any

from fastapi import FastAPI, UploadFile, File, HTTPException, Query
from fastapi.responses import FileResponse, JSONResponse

import ripdoc

# Optional competitor libraries for benchmarking
_HAS_PDFPLUMBER = False
_HAS_PYMUPDF = False
_HAS_PDFMINER = False

try:
    import pdfplumber
    _HAS_PDFPLUMBER = True
except ImportError:
    pass

try:
    import pymupdf
    _HAS_PYMUPDF = True
except ImportError:
    try:
        import fitz as pymupdf
        _HAS_PYMUPDF = True
    except ImportError:
        pass

try:
    from pdfminer.high_level import extract_text as pdfminer_extract_text
    _HAS_PDFMINER = True
except ImportError:
    pass

app = FastAPI(title="Ripdoc Visualizer")


def _timed(fn: Callable[[], Any]) -> tuple[Any, float]:
    """Run fn() and return (result, elapsed_ms)."""
    t0 = time.perf_counter()
    result = fn()
    elapsed = (time.perf_counter() - t0) * 1000
    return result, round(elapsed, 3)

# In-memory state for the currently loaded PDF
_state: dict = {
    "pdf": None,
    "pages": [],
    "path": None,
    "raw_bytes": None,
}


@app.post("/api/upload")
async def upload_pdf(file: UploadFile = File(...)):
    """Accept a multipart PDF upload, store it, return page count + metadata."""
    if not file.filename or not file.filename.lower().endswith(".pdf"):
        raise HTTPException(status_code=400, detail="Only PDF files are accepted")

    data = await file.read()

    # Write to a temp file so we can serve it back for pdf.js
    tmp = tempfile.NamedTemporaryFile(delete=False, suffix=".pdf")
    tmp.write(data)
    tmp.close()

    try:
        pdf = ripdoc.PDF.from_bytes(data)
    except Exception as e:
        os.unlink(tmp.name)
        raise HTTPException(status_code=400, detail=f"Failed to parse PDF: {e}")

    # Clean up previous temp file
    if _state["path"] and os.path.exists(_state["path"]):
        try:
            os.unlink(_state["path"])
        except OSError:
            pass

    _state["pdf"] = pdf
    _state["pages"] = pdf.pages
    _state["path"] = tmp.name
    _state["raw_bytes"] = data

    return {
        "page_count": pdf.page_count,
        "metadata": pdf.metadata,
        "filename": file.filename,
    }


@app.get("/api/pdf-file")
async def get_pdf_file():
    """Return raw PDF bytes for pdf.js to render."""
    if not _state["path"]:
        raise HTTPException(status_code=404, detail="No PDF uploaded")
    return FileResponse(_state["path"], media_type="application/pdf")


def _get_page(n: int):
    """Get a page by 1-indexed number."""
    pages = _state["pages"]
    if not pages:
        raise HTTPException(status_code=404, detail="No PDF uploaded")
    if n < 1 or n > len(pages):
        raise HTTPException(status_code=404, detail=f"Page {n} not found (1-{len(pages)})")
    return pages[n - 1]


@app.get("/api/pages/{n}")
async def page_info(n: int):
    """Return basic page metadata."""
    page = _get_page(n)
    return {
        "page_number": page.page_number,
        "width": page.width,
        "height": page.height,
        "char_count": len(page.chars),
        "bbox": page.bbox,
    }


@app.get("/api/pages/{n}/text")
async def page_text(n: int, layout: bool = Query(False)):
    """Return extracted text."""
    page = _get_page(n)
    text, timing_ms = _timed(lambda: page.extract_text(layout=layout))
    return {"text": text, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/chars")
async def page_chars(n: int):
    """Return all character objects."""
    page = _get_page(n)
    data, timing_ms = _timed(lambda: page.chars)
    return {"data": data, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/words")
async def page_words(n: int):
    """Return word objects."""
    page = _get_page(n)
    data, timing_ms = _timed(lambda: page.extract_words())
    return {"data": data, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/lines")
async def page_lines(n: int):
    """Return line objects."""
    page = _get_page(n)
    data, timing_ms = _timed(lambda: page.lines)
    return {"data": data, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/rects")
async def page_rects(n: int):
    """Return rect objects."""
    page = _get_page(n)
    data, timing_ms = _timed(lambda: page.rects)
    return {"data": data, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/edges")
async def page_edges(n: int):
    """Return edge objects (lines + rect edges)."""
    page = _get_page(n)
    data, timing_ms = _timed(lambda: page.edges)
    return {"data": data, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/tables")
async def page_tables(n: int):
    """Return detected tables with bbox, dimensions, grid data, and HTML."""
    page = _get_page(n)

    def _find():
        tables = page.find_tables()
        result = []
        for t in tables:
            result.append({
                "bbox": {"x0": t.bbox[0], "top": t.bbox[1], "x1": t.bbox[2], "bottom": t.bbox[3]},
                "row_count": t.row_count,
                "col_count": t.col_count,
                "grid": t.extract(),
                "html": t.to_html(),
            })
        return result

    data, timing_ms = _timed(_find)
    return {"data": data, "timing_ms": timing_ms}


@app.get("/api/pages/{n}/search")
async def page_search(n: int, q: str = Query(...)):
    """Search for text on the page."""
    page = _get_page(n)
    if not q:
        return {"data": [], "timing_ms": 0}
    data, timing_ms = _timed(lambda: page.search(q))
    return {"data": data, "timing_ms": timing_ms}


# ---------------------------------------------------------------------------
# Benchmark endpoints
# ---------------------------------------------------------------------------

@app.get("/api/benchmark/libraries")
async def benchmark_libraries():
    """Return which competitor libraries are available."""
    available = ["ripdoc"]
    if _HAS_PDFPLUMBER:
        available.append("pdfplumber")
    if _HAS_PYMUPDF:
        available.append("pymupdf")
    if _HAS_PDFMINER:
        available.append("pdfminer")
    return {"available": available}


def _median_ms(fn: Callable[[], Any], iterations: int) -> float:
    """Run fn() `iterations` times, return median elapsed ms."""
    times = []
    for _ in range(iterations):
        t0 = time.perf_counter()
        fn()
        times.append((time.perf_counter() - t0) * 1000)
    return round(statistics.median(times), 3)


@app.post("/api/benchmark/{n}")
def run_benchmark(n: int, iterations: int = Query(3)):
    """Benchmark ripdoc vs competitor libraries on page n.

    Each iteration includes opening/parsing the PDF from scratch so we measure
    the full pipeline (parse + extract), not just returning cached data.
    Sync def so FastAPI runs it in a threadpool (competitors may block).
    """
    if not _state["raw_bytes"]:
        raise HTTPException(status_code=404, detail="No PDF uploaded")

    raw = _state["raw_bytes"]
    path = _state["path"]
    page_idx = n - 1  # 0-indexed for competitors

    results: dict[str, dict[str, float]] = {}

    # --- ripdoc (open from bytes + single page extract each iteration) ---
    page_num = n  # 1-indexed for ripdoc

    def ripdoc_extract_text():
        pdf = ripdoc.PDF.from_bytes(raw)
        pdf.page(page_num).extract_text()

    def ripdoc_extract_words():
        pdf = ripdoc.PDF.from_bytes(raw)
        pdf.page(page_num).extract_words()

    def ripdoc_find_tables():
        pdf = ripdoc.PDF.from_bytes(raw)
        pdf.page(page_num).find_tables()

    def ripdoc_chars():
        pdf = ripdoc.PDF.from_bytes(raw)
        _ = pdf.page(page_num).chars

    results["ripdoc"] = {
        "extract_text": _median_ms(ripdoc_extract_text, iterations),
        "extract_words": _median_ms(ripdoc_extract_words, iterations),
        "find_tables": _median_ms(ripdoc_find_tables, iterations),
        "chars": _median_ms(ripdoc_chars, iterations),
    }

    # --- pdfplumber (open from file + extract each iteration) ---
    if _HAS_PDFPLUMBER:
        def pp_extract_text():
            with pdfplumber.open(path) as pdf:
                pdf.pages[page_idx].extract_text()

        def pp_extract_words():
            with pdfplumber.open(path) as pdf:
                pdf.pages[page_idx].extract_words()

        def pp_find_tables():
            with pdfplumber.open(path) as pdf:
                pdf.pages[page_idx].find_tables()

        def pp_chars():
            with pdfplumber.open(path) as pdf:
                _ = pdf.pages[page_idx].chars

        results["pdfplumber"] = {
            "extract_text": _median_ms(pp_extract_text, iterations),
            "extract_words": _median_ms(pp_extract_words, iterations),
            "find_tables": _median_ms(pp_find_tables, iterations),
            "chars": _median_ms(pp_chars, iterations),
        }

    # --- pymupdf (open from file + extract each iteration) ---
    if _HAS_PYMUPDF:
        def mu_extract_text():
            doc = pymupdf.open(path)
            doc[page_idx].get_text()
            doc.close()

        def mu_extract_words():
            doc = pymupdf.open(path)
            doc[page_idx].get_text("words")
            doc.close()

        results["pymupdf"] = {
            "extract_text": _median_ms(mu_extract_text, iterations),
            "extract_words": _median_ms(mu_extract_words, iterations),
        }

    # --- pdfminer (always parses full doc from scratch) ---
    if _HAS_PDFMINER:
        results["pdfminer"] = {
            "extract_text": _median_ms(lambda: pdfminer_extract_text(path), iterations),
        }

    return results
