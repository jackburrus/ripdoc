import { useState } from "react";

const MOCK_KEY = "rip_sk_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6";

export default function ApiKeyCard() {
  const [revealed, setRevealed] = useState(false);
  const [copied, setCopied] = useState(false);

  const displayKey = revealed
    ? MOCK_KEY
    : MOCK_KEY.slice(0, 7) + "\u2022".repeat(24);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(MOCK_KEY);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="dash-card">
      <div className="dash-card-title">API Key</div>
      <div className="dash-key-row">
        <div className="dash-key-value">{displayKey}</div>
      </div>
      <div className="dash-key-actions">
        <button className="dash-key-btn" onClick={() => setRevealed(!revealed)}>
          {revealed ? "Hide" : "Reveal"}
        </button>
        <button className="dash-key-btn" onClick={handleCopy}>
          {copied ? "Copied!" : "Copy"}
        </button>
        <button className="dash-key-btn" onClick={() => alert("Regenerate is a mock action")}>
          Regenerate
        </button>
      </div>
    </div>
  );
}
