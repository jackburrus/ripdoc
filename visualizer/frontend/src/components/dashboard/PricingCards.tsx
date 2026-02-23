const TIERS = [
  {
    name: "Free",
    price: "$0",
    period: "forever",
    features: [
      "100 pages / month",
      "Text extraction",
      "Table extraction",
      "Community support",
    ],
    featured: false,
    ctaLabel: "Get started",
  },
  {
    name: "Pro",
    price: "$29",
    period: "/month",
    features: [
      "10,000 pages / month",
      "All extraction modes",
      "Layout preservation",
      "Priority support",
      "Batch processing",
    ],
    featured: true,
    ctaLabel: "Start free trial",
  },
  {
    name: "Enterprise",
    price: "Custom",
    period: "contact us",
    features: [
      "Unlimited pages",
      "Dedicated infrastructure",
      "SLA guarantee",
      "Custom integrations",
      "On-premise option",
    ],
    featured: false,
    ctaLabel: "Contact sales",
  },
];

export default function PricingCards() {
  return (
    <div className="dash-card" style={{ gridColumn: "1 / -1" }}>
      <div className="dash-card-title">Pricing</div>
      <div className="pricing-grid">
        {TIERS.map((tier) => (
          <div
            key={tier.name}
            className={`pricing-card${tier.featured ? " pricing-featured" : ""}`}
          >
            <div className="pricing-name">{tier.name}</div>
            <div className="pricing-price">{tier.price}</div>
            <div className="pricing-period">{tier.period}</div>
            <ul className="pricing-features">
              {tier.features.map((f) => (
                <li key={f}>{f}</li>
              ))}
            </ul>
            <button
              className={`pricing-cta ${
                tier.featured ? "pricing-cta-primary" : "pricing-cta-secondary"
              }`}
              onClick={() => alert(`${tier.name} plan — coming soon!`)}
            >
              {tier.ctaLabel}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
