import type { SessionListItem } from "../lib/api";

type SessionTableProps = {
  sessions: SessionListItem[];
  selectedSessionId?: string;
};

export function SessionTable({
  sessions,
  selectedSessionId
}: SessionTableProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">Session Explorer</p>
          <h2>Retention-first queue</h2>
        </div>
        <p className="panel-copy">
          Review title quality, progress, and recency before exporting or
          deleting anything.
        </p>
      </div>

      <div className="table-shell">
        <table className="data-table">
          <thead>
            <tr>
              <th scope="col">Session</th>
              <th scope="col">Assistant</th>
              <th scope="col">Progress</th>
              <th scope="col">Value</th>
              <th scope="col">Last Activity</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((session) => (
              <tr
                className={
                  session.sessionId === selectedSessionId ? "is-selected" : ""
                }
                key={session.sessionId}
              >
                <td>
                  <a
                    className="session-link"
                    href={`#/sessions/${session.sessionId}`}
                  >
                    {session.title}
                  </a>
                  <div className="session-meta">{session.environment}</div>
                </td>
                <td>{session.assistant}</td>
                <td>
                  <span className="progress-badge">
                    {session.progressState}
                  </span>
                  <span className="progress-percent">
                    {session.progressPercent}%
                  </span>
                </td>
                <td>{session.valueScore}</td>
                <td>{session.lastActivityAt}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
