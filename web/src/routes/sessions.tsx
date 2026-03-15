import { useDeferredValue, useState } from "react";

import { SessionDetail } from "../components/session-detail";
import { SessionTable } from "../components/session-table";
import type { SessionDetailRecord } from "../lib/api";

type SessionsRouteProps = {
  sessions: SessionDetailRecord[];
  selectedSessionId?: string;
};

export function SessionsRoute({
  sessions,
  selectedSessionId
}: SessionsRouteProps) {
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
          Search sessions
        </label>
        <input
          className="search-input"
          id="session-search"
          onChange={(event) => setQuery(event.target.value)}
          placeholder="topic, project, assistant, risk"
          type="search"
          value={query}
        />
      </section>

      <div className="content-grid">
        <SessionTable
          selectedSessionId={selectedSession?.sessionId}
          sessions={filteredSessions}
        />
        <SessionDetail session={selectedSession} />
      </div>
    </section>
  );
}
