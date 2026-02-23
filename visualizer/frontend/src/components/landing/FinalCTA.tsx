import { Link } from "react-router-dom";

export default function FinalCTA() {
  return (
    <section className="final-cta">
      <div className="final-cta-content">
        <h2>
          Supercharge your<br />
          PDF extraction
        </h2>
        <div className="cta-buttons">
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
