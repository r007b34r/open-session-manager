import type { KeyboardEvent } from "react";

import type { SessionListItem } from "../lib/api";
import type { SessionSearchReason } from "../lib/session-search";
import { useI18n } from "../lib/i18n";

type SessionTableProps = {
  sessions: SessionListItem[];
  selectedSessionId?: string;
  onSelectSession?: (sessionId: string) => void;
  searchSnippets?: ReadonlyMap<string, string>;
  searchMatchReasons?: ReadonlyMap<string, SessionSearchReason[]>;
};

export function SessionTable({
  sessions,
  selectedSessionId,
  onSelectSession,
  searchSnippets,
  searchMatchReasons
}: SessionTableProps) {
  const { copy, translateProgressState } = useI18n();

  const selectSession = (sessionId: string) => {
    onSelectSession?.(sessionId);
  };

  const handleRowKeyDown = (
    event: KeyboardEvent<HTMLTableRowElement>,
    sessionId: string
  ) => {
    if (event.key !== "Enter" && event.key !== " ") {
      return;
    }

    event.preventDefault();
    selectSession(sessionId);
  };

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
        {sessions.length === 0 ? (
          <div className="table-empty-state">
            <h3>{copy.sessionTable.emptyTitle}</h3>
            <p className="panel-copy">{copy.sessionTable.emptyBody}</p>
          </div>
        ) : (
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
              {sessions.map((session) => {
                const isSelected = session.sessionId === selectedSessionId;
                const snippet = searchSnippets?.get(session.sessionId);
                const matchReasons = searchMatchReasons?.get(session.sessionId) ?? [];

                return (
                  <tr
                    className={isSelected ? "is-selected" : ""}
                    key={session.sessionId}
                    onClick={() => selectSession(session.sessionId)}
                    onKeyDown={(event) => handleRowKeyDown(event, session.sessionId)}
                    tabIndex={0}
                  >
                    <td>
                      <button
                        aria-pressed={isSelected}
                        className={
                          isSelected
                            ? "session-row-button is-selected"
                            : "session-row-button"
                        }
                        onClick={(event) => {
                          event.stopPropagation();
                          selectSession(session.sessionId);
                        }}
                        type="button"
                      >
                        <span className="session-link">{session.title}</span>
                        <span className="session-meta-stack">
                          <span className="session-meta">{session.environment}</span>
                          <span className="session-meta session-meta-id">
                            {shortenSessionId(session.sessionId)}
                          </span>
                        </span>
                        {snippet ? (
                          <span className="session-search-snippet">{snippet}</span>
                        ) : null}
                        {matchReasons.length > 0 ? (
                          <span className="session-search-reasons">
                            {matchReasons.map((reason) => (
                              <span className="badge badge-neutral" key={reason}>
                                {copy.sessions.matchReasonLabels[reason] ?? reason}
                              </span>
                            ))}
                          </span>
                        ) : null}
                      </button>
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
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </section>
  );
}

function shortenSessionId(value: string) {
  if (value.length <= 14) {
    return value;
  }

  return `${value.slice(0, 8)}...${value.slice(-4)}`;
}
