const STEPS = [
  { label: "PDF Input", desc: "Your document" },
  { label: "lopdf Parser", desc: "Rust PDF parser" },
  { label: "Content Stream", desc: "Operator interpreter" },
  { label: "Object Extraction", desc: "Chars, words, rects" },
  { label: "Analysis", desc: "Tables, layout, search" },
  { label: "Output", desc: "Text, JSON, HTML, CSV" },
];

export default function HowItWorks() {
  return (
    <section className="landing-section section-light">
      <div className="landing-content">
        <h2 className="landing-heading text-center">How it works</h2>
        <p className="landing-subheading text-center">
          Six-stage pipeline from raw PDF bytes to structured output
        </p>
        <div className="landing-pipeline">
          {STEPS.map((step, i) => (
            <div key={step.label} className="pipeline-step">
              <div className="pipeline-number">{i + 1}</div>
              <div className="pipeline-label">{step.label}</div>
              <div className="pipeline-desc">{step.desc}</div>
              {i < STEPS.length - 1 && <div className="pipeline-arrow" />}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
