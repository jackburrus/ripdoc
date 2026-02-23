# ripdoc

**Rust-native PDF extraction. 73x faster than pdfplumber.**

ripdoc is a drop-in replacement for [pdfplumber](https://github.com/jsvine/pdfplumber) built entirely in Rust with Python bindings via PyO3. Extract text, tables, words, and more from PDFs at speeds that feel instant.

```
$ python bench.py report.pdf (200 pages)

ripdoc       0.16s  ██
pymupdf      5.97s  ████████████████████████████████████████
pdfplumber  11.63s  █████████████████████████████████████████████████████████████████████████
pdfminer    15.55s  ██████████████████████████████████████████████████████████████████████████████████████████████████
```

## Install

```bash
pip install ripdoc
```

Requires Python 3.8+. Pre-built wheels for macOS (arm64). Other platforms build from source (requires Rust toolchain).

## Quick start

```python
import ripdoc

pdf = ripdoc.open("report.pdf")

for page in pdf.pages:
    # Extract text
    text = page.extract_text()

    # Extract with layout preservation
    text = page.extract_text(layout=True)

    # Extract words with bounding boxes
    words = page.extract_words()

    # Extract tables
    tables = page.extract_tables()

    # Search for text
    results = page.search("revenue")
```

## Drop-in pdfplumber replacement

Swap one import — everything else stays the same:

```python
# Before
import pdfplumber
pdf = pdfplumber.open("report.pdf")

# After
import ripdoc as pdfplumber
pdf = pdfplumber.open("report.pdf")
```

Or use the explicit compat module:

```python
import ripdoc.compat as pdfplumber
```

## API

### `ripdoc.open(path) -> PDF`

Open a PDF file. Also supports `PDF.from_bytes(bytes)`.

### `PDF`

| Property / Method | Description |
|---|---|
| `pdf.pages` | List of `Page` objects |
| `pdf.page_count` | Number of pages |
| `pdf.metadata` | Document metadata dict |
| `pdf.page(n)` | Get page by number (1-indexed) |

### `Page`

| Property / Method | Description |
|---|---|
| `page.extract_text(layout=False)` | Extract text, optionally preserving spatial layout |
| `page.extract_words()` | Words with bounding boxes (`x0`, `top`, `x1`, `bottom`) |
| `page.extract_tables()` | Tables as list of row lists |
| `page.extract_table()` | Largest table on the page |
| `page.find_tables()` | Table objects with metadata |
| `page.search(query)` | Find text matches with positions |
| `page.chars` | Individual characters with font info |
| `page.lines` | Line segments |
| `page.rects` | Rectangles |
| `page.edges` | Edges (used for table detection) |
| `page.crop(bbox)` | Crop to bounding box `(x0, top, x1, bottom)` |
| `page.within_bbox(bbox)` | Filter objects within bounding box |
| `page.width` / `page.height` | Page dimensions in points |
| `page.page_number` | 1-indexed page number |

## Architecture

```
ripdoc
├── ripdoc-core     Pure Rust library (~5500 LOC)
│   ├── content_stream    PDF operator interpreter
│   ├── fonts/            Encoding, CMap, metrics
│   ├── geometry/         BBox, CTM, clustering
│   ├── text/             Word grouping, layout, search
│   ├── table/            Nurminen/Tabula algorithm
│   └── output/           Markdown, JSON, HTML, CSV
└── ripdoc-python   PyO3 bindings (~450 LOC)
```

Built on [lopdf](https://github.com/nicoulaj/lopdf) for low-level PDF structure parsing. All text extraction, table detection, and layout analysis is implemented from scratch in Rust.

## Features

- **Text extraction** — simple and layout-preserving modes
- **Table detection** — Nurminen/Tabula algorithm with merged cell support
- **Search** — full-text search with bounding box positions
- **Reading order** — XY-cut algorithm + tagged PDF structure tree
- **Output formats** — Markdown, JSON, HTML, CSV
- **Spatial queries** — crop, within_bbox, character-level access
- **pdfplumber compatible** — same API, same patterns

## Development

```bash
# Build and install locally
cd crates/ripdoc-python
maturin develop --release

# Run tests
cargo test

# Type check the visualizer frontend
cd visualizer/frontend && npx tsc --noEmit
```

## License

MIT OR Apache-2.0
