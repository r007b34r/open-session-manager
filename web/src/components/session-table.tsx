import type { SessionListItem } from "../lib/api";
import { useI18n } from "../lib/i18n";

type SessionTableProps = {
  sessions: SessionListItem[];
  selectedSessionId?: string;
};

export function SessionTable({
  sessions,
  selectedSessionId
}: SessionTableProps) {
  const { copy, translateProgressState } = useI18n();

  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.sessionTable.kicker}</p>
          <h2>{copy.sessionTable.title}</h2>
        </div>
        <p className="panel-copy">{copy.sessionTable.description}</p>
      </div>

      <div className="table-shell">
        <table className="data-table">
          <thead>
            <tr>
              <th scope="col">{copy.sessionTable.columns.session}</th>
              <th scope="col">{copy.sessionTable.columns.assistant}</th>
              <th scope="col">{copy.sessionTable.columns.progress}</th>
              <th scope="col">{copy.sessionTable.columns.value}</th>
              <th scope="col">{copy.sessionTable.columns.lastActivity}</th>
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
                    href={`#/sessions/${encodeURIComponent(session.sessionId)}`}
                  >
                    {session.title}
                  </a>
                  <div className="session-meta">{session.environment}</div>
                </td>
                <td>{session.assistant}</td>
                <td>
                  <span className="progress-badge">
                    {translateProgressState(session.progressState)}
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
