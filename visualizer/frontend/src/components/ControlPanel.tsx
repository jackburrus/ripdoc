import type { LayerName, LayerState } from "../types";

interface Props {
  layers: LayerState;
  counts: Record<LayerName, number>;
  timings: Record<string, number>;
  onToggle: (layer: LayerName) => void;
}

const LAYER_CONFIG: { key: LayerName; label: string; color: string }[] = [
  { key: "words", label: "Words", color: "#0064ff" },
  { key: "chars", label: "Chars", color: "#ff0000" },
  { key: "tables", label: "Tables", color: "#c800c8" },
  { key: "lines", label: "Lines", color: "#00b400" },
  { key: "rects", label: "Rects", color: "#ff8c00" },
  { key: "edges", label: "Edges", color: "#009696" },
];

function formatTiming(ms: number): string {
  if (ms < 1) return `${(ms * 1000).toFixed(0)}\u00B5s`;
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

export default function ControlPanel({ layers, counts, timings, onToggle }: Props) {
  return (
    <div className="control-panel">
      <h3>Layers</h3>
      {LAYER_CONFIG.map(({ key, label, color }) => (
        <label key={key} className="layer-toggle">
          <input
            type="checkbox"
            checked={layers[key]}
            onChange={() => onToggle(key)}
          />
          <span className="layer-swatch" style={{ backgroundColor: color }} />
          {label}
          {counts[key] > 0 && (
            <span className="layer-count">({counts[key]})</span>
          )}
          {timings[key] !== undefined && (
            <span className="layer-timing">{formatTiming(timings[key])}</span>
          )}
        </label>
      ))}
    </div>
  );
}
