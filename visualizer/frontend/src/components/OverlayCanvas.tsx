import type {
  Char,
  Word,
  Line,
  Rect,
  Edge,
  TableData,
  SearchMatch,
  LayerState,
} from "../types";

interface Props {
  width: number;
  height: number;
  scale: number;
  layers: LayerState;
  chars: Char[];
  words: Word[];
  lines: Line[];
  rects: Rect[];
  edges: Edge[];
  tables: TableData[];
  searchResults: SearchMatch[];
}

export default function OverlayCanvas({
  width,
  height,
  scale,
  layers,
  chars,
  words,
  lines,
  rects,
  edges,
  tables,
  searchResults,
}: Props) {
  const s = scale; // shorthand

  return (
    <svg
      className="overlay-svg"
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
    >
      {/* Search highlights (yellow, behind everything) */}
      {layers.search &&
        searchResults.map((m, i) => (
          <rect
            key={`search-${i}`}
            x={m.x0 * s}
            y={m.top * s}
            width={(m.x1 - m.x0) * s}
            height={(m.bottom - m.top) * s}
            fill="rgba(255, 255, 0, 0.35)"
            stroke="rgba(255, 200, 0, 0.8)"
            strokeWidth={1}
          />
        ))}

      {/* Chars (red, semi-transparent fill) */}
      {layers.chars &&
        chars.map((c, i) => (
          <rect
            key={`char-${i}`}
            x={c.x0 * s}
            y={c.top * s}
            width={(c.x1 - c.x0) * s}
            height={(c.bottom - c.top) * s}
            fill="rgba(255, 0, 0, 0.12)"
            stroke="rgba(255, 0, 0, 0.5)"
            strokeWidth={0.5}
          />
        ))}

      {/* Words (blue outlines) */}
      {layers.words &&
        words.map((w, i) => (
          <rect
            key={`word-${i}`}
            x={w.x0 * s}
            y={w.top * s}
            width={(w.x1 - w.x0) * s}
            height={(w.bottom - w.top) * s}
            fill="none"
            stroke="rgba(0, 100, 255, 0.6)"
            strokeWidth={1}
          />
        ))}

      {/* Lines (green) */}
      {layers.lines &&
        lines.map((l, i) => (
          <line
            key={`line-${i}`}
            x1={l.x0 * s}
            y1={l.top * s}
            x2={l.x1 * s}
            y2={l.bottom * s}
            stroke="rgba(0, 180, 0, 0.7)"
            strokeWidth={Math.max(1, (l.width || 1) * s)}
          />
        ))}

      {/* Rects (orange outlines) */}
      {layers.rects &&
        rects.map((r, i) => (
          <rect
            key={`rect-${i}`}
            x={r.x0 * s}
            y={r.top * s}
            width={(r.x1 - r.x0) * s}
            height={(r.bottom - r.top) * s}
            fill="rgba(255, 165, 0, 0.08)"
            stroke="rgba(255, 140, 0, 0.7)"
            strokeWidth={1}
          />
        ))}

      {/* Edges (teal hairlines) */}
      {layers.edges &&
        edges.map((e, i) => (
          <line
            key={`edge-${i}`}
            x1={e.x0 * s}
            y1={e.top * s}
            x2={e.x1 * s}
            y2={e.bottom * s}
            stroke="rgba(0, 150, 150, 0.6)"
            strokeWidth={1}
          />
        ))}

      {/* Tables (magenta, thick border) */}
      {layers.tables &&
        tables.map((t, i) => (
          <rect
            key={`table-${i}`}
            x={t.bbox.x0 * s}
            y={t.bbox.top * s}
            width={(t.bbox.x1 - t.bbox.x0) * s}
            height={(t.bbox.bottom - t.bbox.top) * s}
            fill="rgba(200, 0, 200, 0.06)"
            stroke="rgba(200, 0, 200, 0.8)"
            strokeWidth={2}
          />
        ))}
    </svg>
  );
}
