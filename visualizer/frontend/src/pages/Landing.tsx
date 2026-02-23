import { useEffect, useRef, useState, useCallback } from "react";
import { Link } from "react-router-dom";
import ExtractionRace from "../components/landing/ExtractionRace";
import "./Landing.css";

const CODE = [
  { t: "kw", v: "import" }, { t: "fn", v: " ripdoc" }, { t: "br" },
  { t: "br" },
  { t: "plain", v: "pdf = " }, { t: "fn", v: "ripdoc" }, { t: "plain", v: "." },
  { t: "fn", v: "open" }, { t: "plain", v: "(" }, { t: "str", v: '"report.pdf"' },
  { t: "plain", v: ")" }, { t: "br" },
  { t: "plain", v: "page = pdf.pages[" }, { t: "num", v: "0" },
  { t: "plain", v: "]" }, { t: "br" },
  { t: "br" },
  { t: "cm", v: "# Extract text — 73× faster" }, { t: "br" },
  { t: "plain", v: "text = page." }, { t: "fn", v: "extract_text" },
  { t: "plain", v: "()" }, { t: "br" },
  { t: "plain", v: "tables = page." }, { t: "fn", v: "extract_tables" },
  { t: "plain", v: "()" },
];

/* ── Hooks ── */

function useReveal(threshold = 0.18) {
  const ref = useRef<HTMLDivElement>(null);
  const [on, setOn] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const obs = new IntersectionObserver(
      ([e]) => { if (e.isIntersecting) { setOn(true); obs.disconnect(); } },
      { threshold },
    );
    obs.observe(el);
    return () => obs.disconnect();
  }, [threshold]);
  return { ref, on };
}

/* ── Landing ── */

export default function Landing() {
  const heroRef = useRef<HTMLElement>(null);
  const [count, setCount] = useState(0);
  const [raceStarted, setRaceStarted] = useState(false);
  const code = useReveal(0.12);

  /* Mouse spotlight */
  useEffect(() => {
    const el = heroRef.current;
    if (!el) return;
    const move = (e: MouseEvent) => {
      const r = el.getBoundingClientRect();
      el.style.setProperty("--mx", `${e.clientX - r.left}px`);
      el.style.setProperty("--my", `${e.clientY - r.top}px`);
    };
    el.addEventListener("mousemove", move);
    return () => el.removeEventListener("mousemove", move);
  }, []);

  /* Counter 0 → 73 */
  useEffect(() => {
    let t0: number;
    const tick = (now: number) => {
      if (!t0) t0 = now;
      const p = Math.min((now - t0) / 2000, 1);
      setCount(Math.round((1 - Math.pow(1 - p, 4)) * 73));
      if (p < 1) requestAnimationFrame(tick);
    };
    const id = setTimeout(() => requestAnimationFrame(tick), 700);
    return () => clearTimeout(id);
  }, []);

  /* Start race after hero animations settle */
  useEffect(() => {
    const id = setTimeout(() => setRaceStarted(true), 1400);
    return () => clearTimeout(id);
  }, []);

  /* Typewriter */
  const [typed, setTyped] = useState(0);
  const startTyping = useCallback(() => {
    let i = 0;
    const iv = setInterval(() => {
      i++;
      setTyped(i);
      if (i >= CODE.length) clearInterval(iv);
    }, 50);
    return () => clearInterval(iv);
  }, []);

  useEffect(() => {
    if (code.on) return startTyping();
  }, [code.on, startTyping]);

  return (
    <div className="landing">

      {/* ═══ HERO ═══ */}
      <section className="hero" ref={heroRef}>
        <div className="grid-bg" aria-hidden="true" />
        <div className="scan-beam" aria-hidden="true" />
        <div className="spotlight" />

        <div className="hero-inner">
          <div className="big-num" aria-label={`${count} times faster`}>
            <span className="digits">{count}</span>
            <span className="x-mark">×</span>
          </div>

          <p className="context">faster than pdfplumber</p>

          <h1 className="h1">Rust-native PDF extraction.</h1>
          <p className="sub">
            Drop-in pdfplumber replacement. Text, tables, layout, search —
            all at lightning speed.
          </p>

          <ExtractionRace started={raceStarted} />

          <div className="hero-cta">
            <div className="actions">
              <Link to="/playground" className="btn-primary">Try Playground</Link>
              <a href="https://github.com/jackburrus/ripdoc" target="_blank" rel="noopener noreferrer" className="btn-ghost">
                GitHub <span className="arrow">→</span>
              </a>
            </div>

            <div className="pip">
              <span className="pip-chr">$</span>
              <code>pip install ripdoc</code>
              <span className="caret" />
            </div>
          </div>
        </div>
      </section>

      {/* ═══ CODE ═══ */}
      <section className="code-sec" ref={code.ref}>
        <div className={`code-wrap${code.on ? " on" : ""}`}>
          <div className="code-left">
            <h2>Drop-in<br />compatible.</h2>
            <p>
              Same API as pdfplumber — swap the import,
              keep everything else.
            </p>
          </div>
          <div className="code-right">
            <div className="card-glow">
              <div className="term">
                <div className="term-bar">
                  <span className="td r" /><span className="td y" /><span className="td g" />
                  <span className="term-name">extract.py</span>
                </div>
                <pre className="term-body">
{CODE.slice(0, typed).map((tok, i) =>
  tok.t === "br" ? <br key={i} /> : <span key={i} className={tok.t === "plain" ? undefined : tok.t}>{tok.v}</span>
)}
                  <span className="tcaret" />
                </pre>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ═══ FOOTER ═══ */}
      <footer className="ft">
        <div className="ft-in">
          <span className="ft-m">RIPDOC</span>
          <nav className="ft-nav">
            <a href="https://github.com/jackburrus/ripdoc" target="_blank" rel="noopener noreferrer">GitHub</a>
            <a href="https://pypi.org/project/ripdoc/" target="_blank" rel="noopener noreferrer">PyPI</a>
            <Link to="/playground">Playground</Link>
          </nav>
        </div>
      </footer>
    </div>
  );
}
