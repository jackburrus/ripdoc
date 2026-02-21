import { useState } from "react";

interface Props {
  simpleText: string;
  layoutText: string;
}

export default function TextPanel({ simpleText, layoutText }: Props) {
  const [mode, setMode] = useState<"simple" | "layout">("simple");

  return (
    <div className="text-panel">
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
      <pre className="text-content">
        {mode === "simple" ? simpleText : layoutText}
      </pre>
    </div>
  );
}
