import { useEffect, useRef, useState } from "react";
import "./ExtractionRace.css";

/*
 * Extraction Race — above-the-fold visual demo
 *
 * Real benchmarks: ripdoc 0.16s vs pdfplumber 11.63s (73x)
 * We animate ripdoc finishing in ~0.35s (slowed from 0.16 so it's visible)
 * and pdfplumber still crawling at its real 11.63s pace.
 */

/* ── Word-level layout data (viewBox 0 0 260 340) ── */

interface WordBox {
  x: number;
  y: number;
  w: number;
  h: number;
}

// Header region
const HEADER: WordBox[] = [
  { x: 20, y: 20, w: 60, h: 12 },
  { x: 84, y: 20, w: 45, h: 12 },
  { x: 133, y: 20, w: 52, h: 12 },
  { x: 20, y: 36, w: 80, h: 7 },
  { x: 104, y: 36, w: 55, h: 7 },
];

// Body paragraph 1
const PARA1: WordBox[] = [
  { x: 20, y: 58, w: 38, h: 6 }, { x: 62, y: 58, w: 52, h: 6 },
  { x: 118, y: 58, w: 44, h: 6 }, { x: 166, y: 58, w: 56, h: 6 },
  { x: 20, y: 68, w: 48, h: 6 }, { x: 72, y: 68, w: 60, h: 6 },
  { x: 136, y: 68, w: 42, h: 6 }, { x: 182, y: 68, w: 38, h: 6 },
  { x: 20, y: 78, w: 55, h: 6 }, { x: 79, y: 78, w: 46, h: 6 },
  { x: 129, y: 78, w: 62, h: 6 }, { x: 195, y: 78, w: 30, h: 6 },
  { x: 20, y: 88, w: 42, h: 6 }, { x: 66, y: 88, w: 50, h: 6 },
  { x: 120, y: 88, w: 38, h: 6 }, { x: 162, y: 88, w: 54, h: 6 },
  { x: 20, y: 98, w: 60, h: 6 }, { x: 84, y: 98, w: 44, h: 6 },
  { x: 132, y: 98, w: 50, h: 6 },
  { x: 20, y: 108, w: 46, h: 6 }, { x: 70, y: 108, w: 56, h: 6 },
  { x: 130, y: 108, w: 40, h: 6 }, { x: 174, y: 108, w: 48, h: 6 },
];

// Table (4 cols x 5 rows)
const TABLE = { x: 20, y: 128, cols: 4, rows: 5, cellW: 55, cellH: 20 };
const TABLE_CELLS: WordBox[] = [];
for (let r = 0; r < TABLE.rows; r++) {
  for (let c = 0; c < TABLE.cols; c++) {
    TABLE_CELLS.push({
      x: TABLE.x + c * TABLE.cellW + 5,
      y: TABLE.y + r * TABLE.cellH + 7,
      w: TABLE.cellW - 14 + (((r + c) % 3) * 4 - 4),
      h: 5,
    });
  }
}

// Body paragraph 2
const PARA2: WordBox[] = [
  { x: 20, y: 240, w: 50, h: 6 }, { x: 74, y: 240, w: 44, h: 6 },
  { x: 122, y: 240, w: 58, h: 6 }, { x: 184, y: 240, w: 40, h: 6 },
  { x: 20, y: 250, w: 42, h: 6 }, { x: 66, y: 250, w: 52, h: 6 },
  { x: 122, y: 250, w: 46, h: 6 }, { x: 172, y: 250, w: 50, h: 6 },
  { x: 20, y: 260, w: 56, h: 6 }, { x: 80, y: 260, w: 38, h: 6 },
  { x: 122, y: 260, w: 60, h: 6 },
  { x: 20, y: 270, w: 44, h: 6 }, { x: 68, y: 270, w: 48, h: 6 },
  { x: 120, y: 270, w: 36, h: 6 }, { x: 160, y: 270, w: 52, h: 6 },
  { x: 20, y: 280, w: 50, h: 6 }, { x: 74, y: 280, w: 42, h: 6 },
];

// All words in scan order (top to bottom)
const ALL_WORDS: (WordBox & { isTable?: boolean })[] = [
  ...HEADER,
  ...PARA1,
  ...TABLE_CELLS.map((c) => ({ ...c, isTable: true })),
  ...PARA2,
];

const TOTAL_WORDS = ALL_WORDS.length;

/* ── Component ── */

export default function ExtractionRace({ started }: { started: boolean }) {
  // How many words each side has revealed
  const [ripdocN, setRipdocN] = useState(0);
  const [plumberN, setPlumberN] = useState(0);

  // Elapsed timers
  const [ripdocMs, setRipdocMs] = useState(0);
  const [plumberMs, setPlumberMs] = useState(0);

  // Done states
  const ripdocDone = ripdocN >= TOTAL_WORDS;
  const plumberDone = plumberN >= TOTAL_WORDS;

  // ripdoc: all words in ~350ms (slowed from real 160ms so you can see it)
  const ripdocStarted = useRef(false);
  useEffect(() => {
    if (!started || ripdocStarted.current) return;
    ripdocStarted.current = true;
    const duration = 350;
    let t0: number;
    let raf: number;
    const tick = (now: number) => {
      if (!t0) t0 = now;
      const elapsed = now - t0;
      const p = Math.min(elapsed / duration, 1);
      // ease-out cubic
      const eased = 1 - Math.pow(1 - p, 3);
      setRipdocN(Math.round(eased * TOTAL_WORDS));
      setRipdocMs(Math.min(Math.round(elapsed * (160 / duration)), 160));
      if (p < 1) raf = requestAnimationFrame(tick);
    };
    // tiny delay so the user sees the "starting" state
    const id = setTimeout(() => { raf = requestAnimationFrame(tick); }, 800);
    return () => { clearTimeout(id); cancelAnimationFrame(raf); };
  }, [started]);

  // pdfplumber: all words would take 11.63s at real speed
  // We show it linearly crawling — it reveals maybe 15-20% in the time you watch
  const plumberStarted = useRef(false);
  useEffect(() => {
    if (!started || plumberStarted.current) return;
    plumberStarted.current = true;
    const duration = 11630; // real time in ms
    let t0: number;
    let raf: number;
    const tick = (now: number) => {
      if (!t0) t0 = now;
      const elapsed = now - t0;
      const p = Math.min(elapsed / duration, 1);
      setPlumberN(Math.round(p * TOTAL_WORDS));
      setPlumberMs(Math.round(elapsed));
      if (p < 1) raf = requestAnimationFrame(tick);
    };
    const id = setTimeout(() => { raf = requestAnimationFrame(tick); }, 800);
    return () => { clearTimeout(id); cancelAnimationFrame(raf); };
  }, [started]);

  return (
    <div className="race" aria-label="Extraction speed comparison">
      <div className="race__cols">
        {/* ripdoc */}
        <div className="race__col">
          <div className="race__header">
            <span className="race__lib race__lib--rip">ripdoc</span>
            <span className={`race__timer${ripdocDone ? " race__timer--done" : ""}`}>
              {ripdocDone ? "0.16s" : `${(ripdocMs / 1000).toFixed(2)}s`}
            </span>
          </div>
          <div className={`race__doc${ripdocDone ? " race__doc--done" : ""}`}>
            {/* scan line */}
            {started && !ripdocDone && (
              <div className="race__scan race__scan--rip" />
            )}
            <DocSvg
              n={ripdocN}
              variant="ripdoc"
            />
            {ripdocDone && <div className="race__flash" />}
          </div>
          <div className="race__foot">
            <span className="race__count">{ripdocN} / {TOTAL_WORDS} elements</span>
            {ripdocDone && (
              <span className="race__badge race__badge--done">complete</span>
            )}
          </div>
        </div>

        {/* VS divider */}
        <div className="race__vs">vs</div>

        {/* pdfplumber */}
        <div className="race__col">
          <div className="race__header">
            <span className="race__lib race__lib--plumber">pdfplumber</span>
            <span className="race__timer">
              {plumberDone ? "11.63s" : `${(plumberMs / 1000).toFixed(1)}s`}
            </span>
          </div>
          <div className="race__doc">
            {started && !plumberDone && (
              <div className="race__scan race__scan--plumber" />
            )}
            <DocSvg
              n={plumberN}
              variant="plumber"
            />
          </div>
          <div className="race__foot">
            <span className="race__count">{plumberN} / {TOTAL_WORDS} elements</span>
            {started && !plumberDone && (
              <span className="race__badge race__badge--slow">extracting...</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

/* ── SVG document with progressive word reveal ── */

function DocSvg({ n, variant }: { n: number; variant: "ripdoc" | "plumber" }) {
  const tableW = TABLE.cols * TABLE.cellW;
  const tableH = TABLE.rows * TABLE.cellH;
  const isRip = variant === "ripdoc";

  return (
    <svg className="race__svg" viewBox="0 0 260 300" fill="none">
      {/* Page background */}
      <rect width="260" height="300" rx="3" fill="#0c0c20" />

      {/* Ghost content (always visible, very dim) */}
      {HEADER.map((w, i) => (
        <rect key={`gh${i}`} x={w.x} y={w.y} width={w.w} height={w.h} rx="2"
          fill="rgba(255,255,255,0.03)" />
      ))}
      {PARA1.map((w, i) => (
        <rect key={`g1${i}`} x={w.x} y={w.y} width={w.w} height={w.h} rx="1"
          fill="rgba(255,255,255,0.02)" />
      ))}
      {/* Table grid */}
      <rect x={TABLE.x} y={TABLE.y} width={tableW} height={tableH} rx="2"
        fill="rgba(255,255,255,0.01)" stroke="rgba(255,255,255,0.04)" strokeWidth="0.5" />
      {Array.from({ length: TABLE.cols - 1 }, (_, i) => (
        <line key={`tc${i}`}
          x1={TABLE.x + (i + 1) * TABLE.cellW} y1={TABLE.y}
          x2={TABLE.x + (i + 1) * TABLE.cellW} y2={TABLE.y + tableH}
          stroke="rgba(255,255,255,0.04)" strokeWidth="0.5" />
      ))}
      {Array.from({ length: TABLE.rows - 1 }, (_, i) => (
        <line key={`tr${i}`}
          x1={TABLE.x} y1={TABLE.y + (i + 1) * TABLE.cellH}
          x2={TABLE.x + tableW} y2={TABLE.y + (i + 1) * TABLE.cellH}
          stroke="rgba(255,255,255,0.04)" strokeWidth="0.5" />
      ))}
      {TABLE_CELLS.map((w, i) => (
        <rect key={`gc${i}`} x={w.x} y={w.y} width={w.w} height={w.h} rx="1"
          fill="rgba(255,255,255,0.02)" />
      ))}
      {PARA2.map((w, i) => (
        <rect key={`g2${i}`} x={w.x} y={w.y} width={w.w} height={w.h} rx="1"
          fill="rgba(255,255,255,0.02)" />
      ))}

      {/* Revealed overlay boxes */}
      {ALL_WORDS.slice(0, n).map((w, i) => (
        <rect key={`w${i}`}
          x={w.x - 1} y={w.y - 1}
          width={w.w + 2} height={w.h + 2}
          rx="2"
          fill={w.isTable
            ? "rgba(52,211,153,0.06)"
            : isRip ? "rgba(129,140,248,0.10)" : "rgba(255,255,255,0.06)"
          }
          stroke={w.isTable
            ? "rgba(52,211,153,0.5)"
            : isRip ? "rgba(129,140,248,0.5)" : "rgba(255,255,255,0.2)"
          }
          strokeWidth="0.75"
          strokeDasharray={w.isTable ? "3 1.5" : "none"}
          className="race__word"
        />
      ))}

      {/* Table detection border (shows when any table cell is revealed) */}
      {n > HEADER.length + PARA1.length && (
        <rect
          x={TABLE.x - 3} y={TABLE.y - 3}
          width={tableW + 6} height={tableH + 6}
          rx="3"
          fill="none"
          stroke={isRip ? "rgba(52,211,153,0.4)" : "rgba(52,211,153,0.15)"}
          strokeWidth="1"
          strokeDasharray="6 3"
          className="race__table-detect"
        />
      )}
    </svg>
  );
}
