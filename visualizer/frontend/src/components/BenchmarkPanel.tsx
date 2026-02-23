import { useState, useEffect, useRef } from "react";
import type { BenchmarkResults } from "../types";
import * as api from "../api";

interface Props {
  currentPage: number;
  autoRun?: boolean;
}

const LIB_COLORS: Record<string, string> = {
  ripdoc: "#6c63ff",
  pdfplumber: "#ff6b6b",
  pymupdf: "#ffd93d",
  pdfminer: "#6bcb77",
};

const OP_LABELS: Record<string, string> = {
  extract_text: "Extract Text",
  extract_words: "Extract Words",
  find_tables: "Find Tables",
  chars: "Get Chars",
};

export default function BenchmarkPanel({ currentPage, autoRun }: Props) {
  const [results, setResults] = useState<BenchmarkResults | null>(null);
  const [running, setRunning] = useState(false);
  const [animate, setAnimate] = useState(false);
  const autoRanRef = useRef(false);

  // Reset results when page changes
  useEffect(() => {
    setResults(null);
    setAnimate(false);
    autoRanRef.current = false;
  }, [currentPage]);

  // Auto-run on first load when autoRun is true
  useEffect(() => {
    if (autoRun && !autoRanRef.current && !running && !results) {
      autoRanRef.current = true;
      handleRun();
    }
  }, [autoRun, currentPage]);

  const handleRun = async () => {
    setRunning(true);
    setAnimate(false);
    setResults(null);
    try {
      const data = await api.runBenchmark(currentPage);
      setResults(data);
      // Trigger animation after a brief paint delay
      requestAnimationFrame(() => {
        requestAnimationFrame(() => setAnimate(true));
      });
    } catch (e) {
      console.error("Benchmark failed:", e);
    } finally {
      setRunning(false);
    }
  };

  // Collect all operations present in results
  const operations = results
    ? Array.from(
        new Set(Object.values(results).flatMap((ops) => Object.keys(ops)))
      ).sort((a, b) => {
        const order = ["extract_text", "extract_words", "find_tables", "chars"];
        return order.indexOf(a) - order.indexOf(b);
      })
    : [];

  return (
    <div className="benchmark-panel">
      <div className="benchmark-header">
        <h3>Benchmark</h3>
        <button
          className="benchmark-run-btn"
          onClick={handleRun}
          disabled={running}
        >
          {running ? (
            <>
              <span className="benchmark-spinner" />
              Running...
            </>
          ) : results ? "Re-run" : "Run Benchmark"}
        </button>
      </div>

      {running && !results && (
        <div className="benchmark-loading">
          Benchmarking ripdoc vs pdfplumber, pymupdf, pdfminer...
        </div>
      )}

      {results && (
        <div className="benchmark-results">
          {operations.map((op) => {
            // Find max timing for this operation across all libraries
            const timingsForOp = Object.entries(results)
              .filter(([, ops]) => ops[op] !== undefined)
              .map(([lib, ops]) => ({ lib, ms: ops[op] }));

            const maxMs = Math.max(...timingsForOp.map((t) => t.ms));
            const ripdocMs = results.ripdoc?.[op];

            return (
              <div key={op} className="benchmark-op">
                <div className="benchmark-op-label">{OP_LABELS[op] || op}</div>
                {timingsForOp.map(({ lib, ms }) => {
                  const pct = maxMs > 0 ? (ms / maxMs) * 100 : 0;
                  const speedup =
                    lib !== "ripdoc" && ripdocMs && ripdocMs > 0
                      ? Math.round(ms / ripdocMs)
                      : null;

                  return (
                    <div key={lib} className="benchmark-bar-row">
                      <span
                        className="benchmark-lib-name"
                        style={{ color: LIB_COLORS[lib] || "#888" }}
                      >
                        {lib}
                      </span>
                      <div className="benchmark-bar-track">
                        <div
                          className="benchmark-bar-fill"
                          style={{
                            backgroundColor: LIB_COLORS[lib] || "#888",
                            width: animate ? `${pct}%` : "0%",
                          }}
                        />
                      </div>
                      <span className="benchmark-ms">{ms.toFixed(1)}ms</span>
                      {speedup !== null && speedup > 1 && (
                        <span className="benchmark-speedup">{speedup}x</span>
                      )}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
