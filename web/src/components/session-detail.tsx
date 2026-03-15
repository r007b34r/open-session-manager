import type { SessionDetailRecord } from "../lib/api";

type SessionDetailProps = {
  session?: SessionDetailRecord;
};

export function SessionDetail({ session }: SessionDetailProps) {
  if (!session) {
    return (
      <section className="panel detail-panel">
        <p className="section-kicker">Session Detail</p>
        <h2>Select a session</h2>
        <p className="panel-copy">
          Choose a row to inspect summary, evidence, and cleanup readiness.
        </p>
      </section>
    );
  }

  return (
    <section className="panel detail-panel">
      <p className="section-kicker">Session Detail</p>
      <h2>{session.title}</h2>
      <p className="detail-summary">{session.summary}</p>

      <div className="detail-grid">
        <div>
          <h3>Context</h3>
          <ul className="detail-list">
            <li>Assistant: {session.assistant}</li>
            <li>Environment: {session.environment}</li>
            <li>Project: {session.projectPath}</li>
            <li>Source: {session.sourcePath}</li>
          </ul>
        </div>

        <div>
          <h3>Signals</h3>
          <ul className="detail-list">
            <li>Progress: {session.progressState}</li>
            <li>Completion: {session.progressPercent}%</li>
            <li>Value score: {session.valueScore}</li>
            <li>Last active: {session.lastActivityAt}</li>
          </ul>
        </div>
      </div>

      <div className="detail-grid">
        <div>
          <h3>Key Artifacts</h3>
          <ul className="detail-list">
            {session.keyArtifacts.map((artifact) => (
              <li key={artifact}>{artifact}</li>
            ))}
          </ul>
        </div>

        <div>
          <h3>Risk Flags</h3>
          <div className="badge-row">
            {session.riskFlags.length === 0 ? (
              <span className="badge badge-safe">no active risk flags</span>
            ) : (
              session.riskFlags.map((flag) => (
                <span className="badge badge-risk" key={flag}>
                  {flag}
                </span>
              ))
            )}
          </div>
          <h3>Topic Labels</h3>
          <div className="badge-row">
            {session.tags.map((tag) => (
              <span className="badge badge-neutral" key={tag}>
                {tag}
              </span>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
