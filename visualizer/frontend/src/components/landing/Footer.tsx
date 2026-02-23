import { Link } from "react-router-dom";

export default function Footer() {
  return (
    <footer className="landing-footer">
      <div className="landing-footer-inner">
        <div className="footer-left">
          <div className="footer-products">
            <div className="footer-product-row">
              <span className="footer-product-name">RIPDOC</span>
              <span className="footer-product-tag">core</span>
            </div>
            <div className="footer-product-row">
              <span className="footer-product-name">RIPDOC</span>
              <span className="footer-product-tag">python</span>
            </div>
          </div>
        </div>
        <div className="footer-columns">
          <div className="footer-col">
            <h4>GitHub</h4>
            <a
              href="https://github.com/jackburrus/ripdoc"
              target="_blank"
              rel="noopener noreferrer"
            >
              Repository
            </a>
            <a
              href="https://github.com/jackburrus/ripdoc/issues"
              target="_blank"
              rel="noopener noreferrer"
            >
              Issues
            </a>
            <a
              href="https://github.com/jackburrus/ripdoc/discussions"
              target="_blank"
              rel="noopener noreferrer"
            >
              Discussions
            </a>
          </div>
          <div className="footer-col">
            <h4>Resources</h4>
            <a
              href="https://github.com/jackburrus/ripdoc"
              target="_blank"
              rel="noopener noreferrer"
            >
              Documentation
            </a>
            <Link to="/playground">Playground</Link>
            <a
              href="https://pypi.org/project/ripdoc/"
              target="_blank"
              rel="noopener noreferrer"
            >
              PyPI
            </a>
          </div>
        </div>
        <div className="footer-brand-logo">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <rect width="48" height="48" rx="12" fill="#231545" />
            <path
              d="M14 14h20v4H18v4h12v4H18v8h-4V14z"
              fill="#b4f034"
            />
          </svg>
        </div>
      </div>
      <div className="footer-bottom">
        <div className="footer-bottom-inner">
          <span className="footer-bottom-license">MIT License &copy; {new Date().getFullYear()}</span>
          <div className="footer-bottom-links">
            <a
              href="https://github.com/jackburrus/ripdoc"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
            <a
              href="https://pypi.org/project/ripdoc/"
              target="_blank"
              rel="noopener noreferrer"
            >
              PyPI
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
