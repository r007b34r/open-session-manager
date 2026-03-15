import { useDeferredValue, useState } from "react";

import { SessionDetail } from "../components/session-detail";
import { SessionTable } from "../components/session-table";
import type { SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type SessionsRouteProps = {
  sessions: SessionDetailRecord[];
  selectedSessionId?: string;
  onExportMarkdown?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionsRoute({
  sessions,
  selectedSessionId,
  onExportMarkdown,
  onSoftDelete
}: SessionsRouteProps) {
  const { copy } = useI18n();
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);

  const filteredSessions = sessions.filter((session) => {
    const haystack = [
      session.title,
      session.assistant,
      session.environment,
      session.summary,
      session.tags.join(" ")
    ]
      .join(" ")
      .toLowerCase();

    return haystack.includes(deferredQuery.trim().toLowerCase());
  });

  const selectedSession =
    filteredSessions.find((session) => session.sessionId === selectedSessionId) ??
    sessions.find((session) => session.sessionId === selectedSessionId) ??
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
          selectedSessionId={selectedSession?.sessionId}
          sessions={filteredSessions}
        />
        <SessionDetail
          onExportMarkdown={onExportMarkdown}
          onSoftDelete={onSoftDelete}
          session={selectedSession}
        />
      </div>
    </section>
  );
}
