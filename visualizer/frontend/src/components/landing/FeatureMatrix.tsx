const FEATURES = [
  { name: "Text extraction", ripdoc: true, pdfplumber: true, pymupdf: true, pdfminer: true },
  { name: "Table extraction", ripdoc: true, pdfplumber: true, pymupdf: false, pdfminer: false },
  { name: "Layout mode", ripdoc: true, pdfplumber: false, pymupdf: true, pdfminer: false },
  { name: "Search", ripdoc: true, pdfplumber: false, pymupdf: true, pdfminer: false },
  { name: "Speed", ripdoc: "50-100x", pdfplumber: "1x", pymupdf: "5-10x", pdfminer: "0.5x" },
  { name: "Language", ripdoc: "Rust", pdfplumber: "Python", pymupdf: "C/Python", pdfminer: "Python" },
  { name: "License", ripdoc: "MIT", pdfplumber: "MIT", pymupdf: "AGPL", pdfminer: "MIT" },
];

const LIBS = ["ripdoc", "pdfplumber", "pymupdf", "pdfminer"] as const;

function Cell({ value }: { value: boolean | string }) {
  if (typeof value === "boolean") {
    return (
      <span className={value ? "feature-yes" : "feature-no"}>
        {value ? "\u2713" : "\u2014"}
      </span>
    );
  }
  return <span>{value}</span>;
}

export default function FeatureMatrix() {
  return (
    <section className="landing-section section-dark">
      <div className="landing-content">
        <h2 className="landing-heading text-center">Feature comparison</h2>
        <p className="landing-subheading text-center">
          How ripdoc stacks up against popular PDF libraries
        </p>
        <div className="landing-table-wrap">
          <table className="landing-feature-table">
            <thead>
              <tr>
                <th>Feature</th>
                {LIBS.map((lib) => (
                  <th key={lib} className={lib === "ripdoc" ? "feature-highlight-col" : ""}>
                    {lib}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {FEATURES.map((feat) => (
                <tr key={feat.name}>
                  <td>{feat.name}</td>
                  {LIBS.map((lib) => (
                    <td key={lib} className={lib === "ripdoc" ? "feature-highlight-col" : ""}>
                      <Cell value={feat[lib]} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
