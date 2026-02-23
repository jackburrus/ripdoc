import { useState } from "react";

const TABS = [
  {
    label: "curl",
    code: `curl -X POST https://api.ripdoc.dev/v1/extract \\
  -H "Authorization: Bearer rip_sk_..." \\
  -F "file=@document.pdf" \\
  -F "output=text"`,
  },
  {
    label: "Python",
    code: `import requests

resp = requests.post(
    "https://api.ripdoc.dev/v1/extract",
    headers={"Authorization": "Bearer rip_sk_..."},
    files={"file": open("document.pdf", "rb")},
    data={"output": "text"},
)
print(resp.json()["text"])`,
  },
];

export default function QuickStart() {
  const [activeTab, setActiveTab] = useState(0);

  return (
    <div className="dash-card">
      <div className="dash-card-title">Quick Start</div>
      <div className="dash-quickstart-tabs">
        {TABS.map((tab, i) => (
          <button
            key={tab.label}
            className={`dash-quickstart-tab${i === activeTab ? " active" : ""}`}
            onClick={() => setActiveTab(i)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <pre className="dash-quickstart-pre">
        <code>{TABS[activeTab].code}</code>
      </pre>
    </div>
  );
}
