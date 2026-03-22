import { useEffect, useState } from "react";

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

const SEARCH_DEBOUNCE_MS = 220;

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
  const [rawQuery, setRawQuery] = useState("");
  const [committedQuery, setCommittedQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const trimmedRawQuery = rawQuery.trim();

  useEffect(() => {
    if (trimmedRawQuery === committedQuery) {
      setIsSearching(false);
      return;
    }

    setIsSearching(true);

    const timeoutId = window.setTimeout(() => {
      setCommittedQuery(trimmedRawQuery);
      setIsSearching(false);
    }, SEARCH_DEBOUNCE_MS);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [committedQuery, trimmedRawQuery]);

  const searchResults = searchSessions(sessions, committedQuery);
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
  const transcriptFocusBySession = new Map(
    searchResults
      .filter((result) => result.focus?.kind === "transcript")
      .map((result) => [result.session.sessionId, result.focus])
  );

  const selectedSession =
    filteredSessions.find((session) => session.sessionId === selectedSessionId) ??
    filteredSessions[0];
  const selectedExportPath = selectedSession
    ? latestMarkdownExportPaths.get(selectedSession.sessionId)
    : undefined;
  const selectedTranscriptFocus =
    selectedSession?.sessionId
      ? transcriptFocusBySession.get(selectedSession.sessionId)
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
          onChange={(event) => setRawQuery(event.target.value)}
          placeholder={copy.sessions.searchPlaceholder}
          type="search"
          value={rawQuery}
        />
        <p className="search-summary">
          {isSearching
            ? copy.sessions.searchSummaryPending
            : committedQuery
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
          transcriptFocus={
            selectedTranscriptFocus?.kind === "transcript"
              ? {
                  highlightIndex: selectedTranscriptFocus.highlightIndex,
                  terms: selectedTranscriptFocus.terms
                }
              : undefined
          }
        />
      </div>
    </section>
  );
}
