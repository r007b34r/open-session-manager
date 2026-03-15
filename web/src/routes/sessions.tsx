import { useDeferredValue, useState } from "react";

import { SessionDetail } from "../components/session-detail";
import { SessionTable } from "../components/session-table";
import type { SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type SessionsRouteProps = {
  sessions: SessionDetailRecord[];
  exportedSessionIds: ReadonlySet<string>;
  selectedSessionId?: string;
  onSelectSession?: (sessionId: string) => void;
  onExportMarkdown?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionsRoute({
  sessions,
  exportedSessionIds,
  selectedSessionId,
  onSelectSession,
  onExportMarkdown,
  onSoftDelete
}: SessionsRouteProps) {
  const { copy } = useI18n();
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);
  const normalizedQuery = deferredQuery.trim().toLowerCase();

  const filteredSessions = sessions.filter((session) => {
    const haystack = [
      session.sessionId,
      session.title,
      session.assistant,
      session.environment,
      session.summary,
      session.projectPath,
      session.sourcePath,
      session.tags.join(" "),
      session.riskFlags.join(" "),
      session.keyArtifacts.join(" "),
      session.transcriptHighlights.map((item) => item.content).join(" "),
      session.todoItems.map((item) => item.content).join(" ")
    ]
      .join(" ")
      .toLowerCase();

    return haystack.includes(normalizedQuery);
  });

  const selectedSession =
    filteredSessions.find((session) => session.sessionId === selectedSessionId) ??
    filteredSessions[0];

  return (
    <section className="route-stack">
      <section className="panel filter-panel">
        <label className="search-label" htmlFor="session-search">
          {copy.sessions.searchLabel}
        </label>
        <input
          className="search-input"
          id="session-search"
          onChange={(event) => setQuery(event.target.value)}
          placeholder={copy.sessions.searchPlaceholder}
          type="search"
          value={query}
        />
      </section>

      <div className="content-grid">
        <SessionTable
          onSelectSession={onSelectSession}
          selectedSessionId={selectedSession?.sessionId}
          sessions={filteredSessions}
        />
        <SessionDetail
          canSoftDelete={
            selectedSession ? exportedSessionIds.has(selectedSession.sessionId) : false
          }
          onExportMarkdown={onExportMarkdown}
          onSoftDelete={onSoftDelete}
          session={selectedSession}
        />
      </div>
    </section>
  );
}
