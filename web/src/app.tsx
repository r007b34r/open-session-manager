const shellStyle: React.CSSProperties = {
  minHeight: "100vh",
  margin: 0,
  display: "grid",
  placeItems: "center",
  background:
    "radial-gradient(circle at top, rgba(40, 106, 72, 0.18), transparent 40%), #f5f1e8",
  color: "#1f2a1f",
  fontFamily: "\"Segoe UI\", sans-serif",
  padding: "2rem"
};

const panelStyle: React.CSSProperties = {
  width: "min(720px, 100%)",
  background: "rgba(255, 252, 247, 0.92)",
  border: "1px solid rgba(31, 42, 31, 0.12)",
  borderRadius: "24px",
  padding: "2rem",
  boxShadow: "0 24px 80px rgba(31, 42, 31, 0.08)"
};

const eyebrowStyle: React.CSSProperties = {
  display: "inline-block",
  marginBottom: "1rem",
  padding: "0.35rem 0.75rem",
  borderRadius: "999px",
  background: "#d8ead9",
  fontSize: "0.85rem",
  fontWeight: 700,
  letterSpacing: "0.04em",
  textTransform: "uppercase"
};

export function App() {
  return (
    <main style={shellStyle}>
      <section style={panelStyle}>
        <span style={eyebrowStyle}>Bootstrap</span>
        <h1 style={{ fontSize: "clamp(2.5rem, 6vw, 4rem)", margin: 0 }}>
          Agent Session Governance
        </h1>
        <p style={{ fontSize: "1.1rem", lineHeight: 1.7, maxWidth: "56ch" }}>
          Local-first control center for inspecting coding-agent sessions,
          configs, and cleanup actions before any destructive change is made.
        </p>
      </section>
    </main>
  );
}
