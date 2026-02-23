import { Link } from "react-router-dom";

export default function Hero() {
  return (
    <section className="hero">
      <div className="hero-content">
        <div className="hero-icon">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <rect width="48" height="48" rx="12" fill="#231545" />
            <path
              d="M14 14h20v4H18v4h12v4H18v8h-4V14z"
              fill="#b4f034"
            />
          </svg>
        </div>
        <span className="hero-badge">RIPDOC</span>
        <h1 className="hero-title">
          Extract at lightspeed
        </h1>
        <p className="hero-subtitle">
          An extremely fast PDF extraction library, written in Rust.
        </p>
        <div className="hero-ctas">
          <Link to="/playground" className="btn-primary">
            Get Started
          </Link>
          <a
            href="https://github.com/jackburrus/ripdoc"
            target="_blank"
            rel="noopener noreferrer"
            className="btn-secondary"
          >
            Browse Docs
          </a>
        </div>
      </div>
    </section>
  );
}
