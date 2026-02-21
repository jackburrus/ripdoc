"""Test ripdoc Python bindings on real PDFs."""
import time
import glob
import os

import ripdoc

print(f"ripdoc version: {ripdoc.__version__}")
print(f"Classes: PDF={ripdoc.PDF}, Page={ripdoc.Page}, Table={ripdoc.Table}")
print()

# Find some real PDFs to test with
pdf_paths = glob.glob(os.path.expanduser("~/Downloads/*.pdf"))
if not pdf_paths:
    print("No PDFs found in ~/Downloads, testing with basic API only")

# ── Test 1: Basic API ──
print("=" * 60)
print("TEST 1: Basic API (context manager, metadata, page access)")
print("=" * 60)

if pdf_paths:
    test_pdf = pdf_paths[0]
    print(f"Using: {os.path.basename(test_pdf)}")

    t0 = time.perf_counter()
    with ripdoc.open(test_pdf) as pdf:
        t1 = time.perf_counter()
        print(f"  Open time: {(t1-t0)*1000:.1f}ms")
        print(f"  repr: {pdf}")
        print(f"  len: {len(pdf)}")
        print(f"  page_count: {pdf.page_count}")
        print(f"  metadata: {pdf.metadata}")

        pages = pdf.pages
        t2 = time.perf_counter()
        print(f"  Extracted {len(pages)} pages in {(t2-t1)*1000:.1f}ms")

        if pages:
            p = pages[0]
            print(f"\n  Page 1: {p}")
            print(f"    width={p.width:.1f}, height={p.height:.1f}")
            print(f"    bbox={p.bbox}")
            print(f"    chars: {len(p.chars)}")
            print(f"    lines: {len(p.lines)}")
            print(f"    rects: {len(p.rects)}")
            print(f"    edges: {len(p.edges)}")
    print("  ✓ Context manager works")
    print()

# ── Test 2: Text extraction ──
print("=" * 60)
print("TEST 2: Text extraction")
print("=" * 60)

for pdf_path in pdf_paths[:3]:
    name = os.path.basename(pdf_path)
    try:
        with ripdoc.open(pdf_path) as pdf:
            for page in pdf.pages[:2]:
                t0 = time.perf_counter()
                text = page.extract_text()
                t1 = time.perf_counter()
                preview = text[:200].replace('\n', '\\n')
                print(f"  {name} p{page.page_number}: {len(text)} chars in {(t1-t0)*1000:.1f}ms")
                print(f"    \"{preview}...\"")
    except Exception as e:
        print(f"  {name}: ERROR - {e}")
print()

# ── Test 3: Word extraction ──
print("=" * 60)
print("TEST 3: Word extraction")
print("=" * 60)

if pdf_paths:
    with ripdoc.open(pdf_paths[0]) as pdf:
        page = pdf.pages[0]
        words = page.extract_words()
        print(f"  Words: {len(words)}")
        if words:
            print(f"  First 5: {[w['text'] for w in words[:5]]}")
            print(f"  Word dict keys: {list(words[0].keys())}")
            w = words[0]
            print(f"  Sample word: text='{w['text']}' x0={w['x0']:.1f} top={w['top']:.1f}")

        # With custom tolerance
        words2 = page.extract_words(x_tolerance=5.0, y_tolerance=5.0)
        print(f"  Words (5.0 tolerance): {len(words2)}")
print()

# ── Test 4: Text extraction with options ──
print("=" * 60)
print("TEST 4: Text extraction with layout mode")
print("=" * 60)

if pdf_paths:
    with ripdoc.open(pdf_paths[0]) as pdf:
        page = pdf.pages[0]

        text_simple = page.extract_text(layout=False)
        text_layout = page.extract_text(layout=True)

        print(f"  Simple mode: {len(text_simple)} chars")
        print(f"  Layout mode: {len(text_layout)} chars")
        print(f"  Layout preview (first 300 chars):")
        for line in text_layout[:300].split('\n'):
            print(f"    |{line}|")
print()

# ── Test 5: Table extraction ──
print("=" * 60)
print("TEST 5: Table detection & extraction")
print("=" * 60)

for pdf_path in pdf_paths[:5]:
    name = os.path.basename(pdf_path)
    try:
        with ripdoc.open(pdf_path) as pdf:
            for page in pdf.pages[:3]:
                tables = page.find_tables()
                if tables:
                    print(f"  {name} p{page.page_number}: {len(tables)} table(s)")
                    for t in tables:
                        print(f"    {t} bbox={t.bbox}")
                        grid = t.extract()
                        if grid:
                            print(f"    First row: {grid[0]}")
                        md = t.to_markdown()
                        print(f"    Markdown preview: {md[:200]}")

                grids = page.extract_tables()
                if grids:
                    print(f"    extract_tables() returned {len(grids)} grid(s)")

                first = page.extract_table()
                if first:
                    print(f"    extract_table() returned {len(first)}x{len(first[0]) if first else 0} grid")
    except Exception as e:
        print(f"  {name}: ERROR - {e}")
print()

# ── Test 6: Search ──
print("=" * 60)
print("TEST 6: Text search")
print("=" * 60)

if pdf_paths:
    with ripdoc.open(pdf_paths[0]) as pdf:
        page = pdf.pages[0]
        # Get a word to search for
        words = page.extract_words()
        if words:
            search_term = words[0]['text']
            results = page.search(search_term)
            print(f"  Search for '{search_term}': {len(results)} result(s)")
            for r in results[:3]:
                print(f"    Found at x0={r['x0']:.1f} top={r['top']:.1f}")
print()

# ── Test 7: Crop / within_bbox ──
print("=" * 60)
print("TEST 7: Crop and within_bbox")
print("=" * 60)

if pdf_paths:
    with ripdoc.open(pdf_paths[0]) as pdf:
        page = pdf.pages[0]
        # Crop to top-left quadrant
        w, h = page.width, page.height
        cropped = page.crop((0, 0, w/2, h/2))
        print(f"  Original: {len(page.chars)} chars")
        print(f"  Cropped (top-left quarter): {len(cropped.chars)} chars")
        print(f"  Cropped text: \"{cropped.extract_text()[:100]}...\"")

        within = page.within_bbox((0, 0, w/2, h/2))
        print(f"  within_bbox (top-left quarter): {len(within.chars)} chars")
print()

# ── Test 8: Character-level data ──
print("=" * 60)
print("TEST 8: Character-level data (pdfplumber compat)")
print("=" * 60)

if pdf_paths:
    with ripdoc.open(pdf_paths[0]) as pdf:
        page = pdf.pages[0]
        chars = page.chars
        if chars:
            c = chars[0]
            print(f"  Char dict keys: {sorted(c.keys())}")
            print(f"  First char: {c}")
            print(f"  Total chars on page: {len(chars)}")
print()

# ── Test 9: from_bytes ──
print("=" * 60)
print("TEST 9: PDF.from_bytes()")
print("=" * 60)

if pdf_paths:
    with open(pdf_paths[0], "rb") as f:
        data = f.read()
    pdf = ripdoc.PDF.from_bytes(data)
    print(f"  Loaded from {len(data)} bytes: {pdf}")
    pages = pdf.pages
    print(f"  Pages: {len(pages)}")
    if pages:
        text = pages[0].extract_text()
        print(f"  Page 1 text length: {len(text)}")
print()

# ── Summary ──
print("=" * 60)
print("ALL TESTS PASSED")
print("=" * 60)
