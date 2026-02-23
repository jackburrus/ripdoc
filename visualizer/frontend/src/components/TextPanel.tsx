import { useState } from "react";

interface Props {
  simpleText: string;
  layoutText: string;
}

export default function TextPanel({ simpleText, layoutText }: Props) {
  const [mode, setMode] = useState<"simple" | "layout">("simple");
  const text = mode === "simple" ? simpleText : layoutText;

  return (
    <div className="text-panel">
      <div className="text-panel-header">
        <h3>Extracted Text</h3>
        <div className="text-panel-tabs">
          <button
            className={mode === "simple" ? "active" : ""}
            onClick={() => setMode("simple")}
          >
            Simple
          </button>
          <button
            className={mode === "layout" ? "active" : ""}
            onClick={() => setMode("layout")}
          >
            Layout
          </button>
        </div>
      </div>
      <pre className="text-content">
        {text || "No text extracted"}
      </pre>
    </div>
  );
}
