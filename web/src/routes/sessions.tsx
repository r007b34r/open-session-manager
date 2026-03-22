import { useDeferredValue, useState } from "react";

import { RuntimePanel } from "../components/runtime-panel";
import { SessionDetail } from "../components/session-detail";
import { SessionTable } from "../components/session-table";
import type { DashboardRuntime, SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";
import { searchSessions } from "../lib/session-search";

type SessionsRouteProps = {
  runtime: DashboardRuntime;
  sessions: SessionDetailRecord[];
  exportedSessionIds: ReadonlySet<string>;
  latestMarkdownExportPaths: ReadonlyMap<string, string>;
  selectedSessionId?: string;
  onSelectSession?: (sessionId: string) => void;
  onSaveExportRoot?: (path: string) => void;
  onResetExportRoot?: () => void;
  onExportMarkdown?: (sessionId: string) => void;
  onResumeSession?: (sessionId: string) => void;
  onContinueSession?: (sessionId: string, prompt: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionsRoute({
  runtime,
  sessions,
  exportedSessionIds,
  latestMarkdownExportPaths,
  selectedSessionId,
  onSelectSession,
  onSaveExportRoot,
  onResetExportRoot,
  onExportMarkdown,
  onResumeSession,
  onContinueSession,
  onSoftDelete
}: SessionsRouteProps) {
  const { copy } = useI18n();
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);
  const trimmedQuery = deferredQuery.trim();
  const searchResults = searchSessions(sessions, trimmedQuery);
  const filteredSessions = searchResults.map((result) => result.session);
  const searchSnippets = new Map(
    searchResults
      .filter((result) => Boolean(result.snippet))
      .map((result) => [result.session.sessionId, result.snippet as string])
  );
  const searchMatchReasons = new Map(
    searchResults
      .filter((result) => result.matchReasons.length > 0)
      .map((result) => [result.session.sessionId, result.matchReasons])
  );

  const selectedSession =
    filteredSessions.find((session) => session.sessionId === selectedSessionId) ??
    filteredSessions[0];
  const selectedExportPath = selectedSession
    ? latestMarkdownExportPaths.get(selectedSession.sessionId)
    : undefined;

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
        <p className="search-summary">
          {trimmedQuery
            ? `${filteredSessions.length} ${copy.sessions.searchSummary}`
            : copy.sessions.searchSummaryEmpty}
        </p>
      </section>

      <RuntimePanel
        onResetExportRoot={onResetExportRoot}
        onSaveExportRoot={onSaveExportRoot}
        runtime={runtime}
      />

      <div className="content-grid">
        <SessionTable
          onSelectSession={onSelectSession}
          searchMatchReasons={searchMatchReasons}
          searchSnippets={searchSnippets}
          selectedSessionId={selectedSession?.sessionId}
          sessions={filteredSessions}
        />
        <SessionDetail
          canSoftDelete={
            selectedSession ? exportedSessionIds.has(selectedSession.sessionId) : false
          }
          exportPath={selectedExportPath}
          onContinueSession={onContinueSession}
          onExportMarkdown={onExportMarkdown}
          onResumeSession={onResumeSession}
          onSoftDelete={onSoftDelete}
          session={selectedSession}
        />
      </div>
    </section>
  );
}
