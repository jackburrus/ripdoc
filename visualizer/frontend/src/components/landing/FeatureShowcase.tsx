function MockWindow({
  filename,
  children,
  reverse,
}: {
  filename?: string;
  children: React.ReactNode;
  reverse?: boolean;
}) {
  return (
    <div className={`mock-window${reverse ? " mock-window--reverse" : ""}`}>
      <div className="mock-titlebar">
        <span className="mock-dot mock-dot--red" />
        <span className="mock-dot mock-dot--yellow" />
        <span className="mock-dot mock-dot--green" />
        {filename && <span className="mock-filename">{filename}</span>}
      </div>
      <div className="mock-body">{children}</div>
    </div>
  );
}

export default function FeatureShowcase() {
  return (
    <section className="features-section">
      {/* Feature 1: Shockingly fast */}
      <div className="feature-row">
        <div className="feature-text">
          <h3>"Shockingly fast"</h3>
          <p>
            ripdoc is 50-100x faster than existing
            tools, extracting sub-second batches
            even on the largest documents.
          </p>
        </div>
        <div className="feature-card">
          <MockWindow filename="benchmark.sh">
            <pre>
              <span className="syn-cm">$ python bench.py report.pdf</span>
              {"\n\n"}
              <span className="syn-fn">ripdoc</span>
              {"      "}
              <span className="syn-num">2.0ms</span>
              {"  "}
              <span className="syn-str">{"██"}</span>
              {"\n"}
              <span className="syn-fn">pymupdf</span>
              {"     "}
              <span className="syn-num">18ms</span>
              {"   "}
              <span className="syn-str">{"████████████"}</span>
              {"\n"}
              <span className="syn-fn">pdfplumber</span>
              {"  "}
              <span className="syn-num">146ms</span>
              {"  "}
              <span className="syn-str">
                {"████████████████████████████████████████"}
              </span>
              {"\n"}
              <span className="syn-fn">pdfminer</span>
              {"    "}
              <span className="syn-num">210ms</span>
              {"  "}
              <span className="syn-str">
                {"████████████████████████████████████████████████████████"}
              </span>
              {"\n\n"}
              <span className="syn-cm">
                {"# ripdoc: 73x faster than pdfplumber"}
              </span>
            </pre>
          </MockWindow>
        </div>
      </div>

      {/* Feature 2: All-in-one */}
      <div className="feature-row feature-row--reversed">
        <div className="feature-text">
          <h3>All-in-one</h3>
          <p>
            Replace dozens of tools with a single
            unified interface. ripdoc supports text,
            tables, layout, search, and multiple output
            formats out of the box.
          </p>
        </div>
        <div className="feature-card">
          <MockWindow filename="extract.py" reverse>
            <pre>
              <span className="syn-kw">import</span>
              <span className="syn-fn"> ripdoc</span>
              {"\n\n"}
              <span className="syn-cm"># Open any PDF</span>
              {"\n"}
              {"pdf = "}
              <span className="syn-fn">ripdoc</span>
              <span className="syn-op">.</span>
              <span className="syn-fn">open</span>
              <span className="syn-op">(</span>
              <span className="syn-str">"report.pdf"</span>
              <span className="syn-op">)</span>
              {"\n"}
              {"page = pdf.pages["}
              <span className="syn-num">0</span>
              {"]"}
              {"\n\n"}
              <span className="syn-cm"># Text, tables, layout, search</span>
              {"\n"}
              {"text = page."}
              <span className="syn-fn">extract_text</span>
              {"()"}
              {"\n"}
              {"tables = page."}
              <span className="syn-fn">extract_tables</span>
              {"()"}
              {"\n"}
              {"layout = page."}
              <span className="syn-fn">extract_text</span>
              {"(layout="}
              <span className="syn-kw">True</span>
              {")"}
              {"\n"}
              {"results = page."}
              <span className="syn-fn">search</span>
              <span className="syn-op">(</span>
              <span className="syn-str">"revenue"</span>
              <span className="syn-op">)</span>
            </pre>
          </MockWindow>
        </div>
      </div>

      {/* Feature 3: Drop-in compatible */}
      <div className="feature-row">
        <div className="feature-text">
          <h3>Drop-in compatible</h3>
          <p>
            Swap your pdfplumber import and
            instantly get 50-100x faster extraction.
            Same API, same patterns, dramatically
            better performance.
          </p>
        </div>
        <div className="feature-card">
          <MockWindow filename="migrate.py">
            <pre>
              <span className="syn-cm"># Before</span>
              {"\n"}
              <span className="syn-kw">import</span>
              <span className="syn-fn"> pdfplumber</span>
              {"\n"}
              {"pdf = "}
              <span className="syn-fn">pdfplumber</span>
              <span className="syn-op">.</span>
              <span className="syn-fn">open</span>
              <span className="syn-op">(</span>
              <span className="syn-str">"report.pdf"</span>
              <span className="syn-op">)</span>
              {"\n\n"}
              <span className="syn-cm"># After — just change the import</span>
              {"\n"}
              <span className="syn-kw">import</span>
              <span className="syn-fn"> ripdoc</span>
              {" "}
              <span className="syn-kw">as</span>
              <span className="syn-fn"> pdfplumber</span>
              {"\n"}
              {"pdf = "}
              <span className="syn-fn">pdfplumber</span>
              <span className="syn-op">.</span>
              <span className="syn-fn">open</span>
              <span className="syn-op">(</span>
              <span className="syn-str">"report.pdf"</span>
              <span className="syn-op">)</span>
              {"\n\n"}
              <span className="syn-cm">{"# Everything else stays the same ✓"}</span>
            </pre>
          </MockWindow>
        </div>
      </div>
    </section>
  );
}
