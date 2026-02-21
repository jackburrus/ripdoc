use ripdoc_core::{Document, TextExtractOptions, TableSettings};

fn main() {
    let path = std::env::args().nth(1).expect("Usage: extract <pdf_path>");

    println!("Opening: {}", path);
    let start = std::time::Instant::now();

    let mut doc = Document::open(&path).expect("Failed to open PDF");

    let open_time = start.elapsed();
    println!("Opened in {:?}", open_time);
    println!("Pages: {}", doc.page_count());
    println!("Metadata: {:?}", doc.metadata());
    println!();

    let page_count = doc.page_count();
    for i in 1..=page_count.min(5) {
        let page_start = std::time::Instant::now();
        let page = doc.page(i).expect("Failed to extract page");
        let page_time = page_start.elapsed();

        println!("=== Page {} ({:.1} x {:.1} pts) — extracted in {:?} ===",
            i, page.width, page.height, page_time);
        println!("  Chars: {}", page.chars.len());
        println!("  Lines: {}", page.lines.len());
        println!("  Rects: {}", page.rects.len());
        println!("  Curves: {}", page.curves.len());

        // Extract text
        let opts = TextExtractOptions::default();
        let text = page.extract_text(&opts);
        let preview: String = text.chars().take(500).collect();
        println!("\n  --- Text (first 500 chars) ---");
        println!("{}", preview);

        // Extract words
        let words = page.words(3.0, 3.0);
        println!("\n  Words: {}", words.len());
        if !words.is_empty() {
            let first_words: Vec<&str> = words.iter().take(10).map(|w| w.text.as_str()).collect();
            println!("  First 10: {:?}", first_words);
        }

        // Try table detection
        let settings = TableSettings::default();
        let tables = ripdoc_core::table::extract::extract_tables(page, &settings);
        println!("\n  Tables found: {}", tables.len());
        for (j, table) in tables.iter().enumerate() {
            println!("    Table {}: {}x{} (bbox: {:.1},{:.1} → {:.1},{:.1})",
                j + 1, table.row_count, table.col_count,
                table.bbox.x0, table.bbox.top, table.bbox.x1, table.bbox.bottom);
            let grid = table.to_grid();
            for (r, row) in grid.iter().enumerate().take(3) {
                let cells: Vec<String> = row.iter()
                    .map(|c| c.as_deref().unwrap_or("").to_string())
                    .collect();
                println!("      Row {}: {:?}", r, cells);
            }
            if grid.len() > 3 {
                println!("      ... ({} more rows)", grid.len() - 3);
            }
        }

        println!();
    }

    let total = start.elapsed();
    println!("Total time: {:?}", total);
}
