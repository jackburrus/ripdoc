import { useEffect, useRef, useState } from "react";

const BENCHMARKS = [
  { lib: "ripdoc", time: "0.16s", highlight: true },
  { lib: "pymupdf", time: "5.97s", highlight: false },
  { lib: "pdfplumber", time: "11.63s", highlight: false },
  { lib: "pdfminer", time: "15.55s", highlight: false },
  { lib: "camelot", time: "45.42s", highlight: false },
  { lib: "tika", time: "68.86s", highlight: false },
];

const TRUSTED = [
  "pydantic",
  "HuggingFace",
  "FastAPI",
  "LangChain",
  "SciPy",
  "pandas",
  "Jupyter",
];

export default function BenchmarkWidget() {
  const ref = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { threshold: 0.2 }
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <section className="benchmark-section" ref={ref}>
      <div className="benchmark-content">
        <div className="benchmark-terminal">
          <div className="terminal-body">
            <div className="benchmark-desc">
              Extracting text from a 200-page PDF document.
            </div>
            <div className="benchmark-command">.extract_text</div>
            <div className={`benchmark-rows ${visible ? "benchmark-rows--visible" : ""}`}>
              {BENCHMARKS.map(({ lib, time, highlight }, i) => (
                <div
                  key={lib}
                  className={`benchmark-row ${highlight ? "benchmark-row--highlight" : ""}`}
                  style={{ transitionDelay: `${i * 80}ms` }}
                >
                  <span className="benchmark-lib">{lib}</span>
                  <span className="benchmark-dots" />
                  <span className="benchmark-time">{time}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      <div className="trusted-section">
        <div className="trusted-label">BUILT FOR THE PYTHON ECOSYSTEM</div>
        <div className="trusted-logos">
          {TRUSTED.map((name) => (
            <span key={name} className="trusted-logo">{name}</span>
          ))}
        </div>
      </div>
    </section>
  );
}
