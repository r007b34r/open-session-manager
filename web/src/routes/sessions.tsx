import { useEffect, useState } from "react";

import { RuntimePanel } from "../components/runtime-panel";
import { SessionDetail } from "../components/session-detail";
import { SessionTable } from "../components/session-table";
import type { DashboardRuntime, SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";
import {
  applySessionFilters,
  DEFAULT_SESSION_FILTERS,
  hasActiveSessionFilters,
  type SessionFilterState
} from "../lib/session-filters";
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
  onAttachSession?: (sessionId: string) => void;
  onExportMarkdown?: (sessionId: string) => void;
  onResumeSession?: (sessionId: string) => void;
  onContinueSession?: (sessionId: string, prompt: string) => void;
  onDetachSession?: (sessionId: string) => void;
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
  onAttachSession,
  onExportMarkdown,
  onResumeSession,
  onContinueSession,
  onDetachSession,
  onSoftDelete
}: SessionsRouteProps) {
  const { copy } = useI18n();
  const [rawQuery, setRawQuery] = useState("");
  const [committedQuery, setCommittedQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [filters, setFilters] = useState<SessionFilterState>(DEFAULT_SESSION_FILTERS);
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
  const filteredSessionIds = new Set(
    applySessionFilters(
      searchResults.map((result) => result.session),
      filters,
      { exportedSessionIds }
    ).map((session) => session.sessionId)
  );
  const filteredResults = searchResults.filter((result) =>
    filteredSessionIds.has(result.session.sessionId)
  );
  const filteredSessions = filteredResults.map((result) => result.session);
  const hasActiveFilters = hasActiveSessionFilters(filters);
  const assistantOptions = uniqueValues(sessions.map((session) => session.assistant));
  const projectOptions = uniqueValues(sessions.map((session) => session.projectPath));
  const searchSnippets = new Map(
    filteredResults
      .filter((result) => Boolean(result.snippet))
      .map((result) => [result.session.sessionId, result.snippet as string])
  );
  const searchMatchReasons = new Map(
    filteredResults
      .filter((result) => result.matchReasons.length > 0)
      .map((result) => [result.session.sessionId, result.matchReasons])
  );
  const transcriptFocusBySession = new Map(
    filteredResults
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
        <div className="session-filter-grid">
          <label className="session-filter-field" htmlFor="session-filter-assistant">
            <span>{copy.sessions.filters.labels.assistant}</span>
            <select
              className="session-filter-select"
              id="session-filter-assistant"
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  assistant: event.target.value
                }))
              }
              value={filters.assistant}
            >
              <option value="all">{copy.sessions.filters.options.allAssistants}</option>
              {assistantOptions.map((assistant) => (
                <option key={assistant} value={assistant}>
                  {assistant}
                </option>
              ))}
            </select>
          </label>
          <label className="session-filter-field" htmlFor="session-filter-project">
            <span>{copy.sessions.filters.labels.project}</span>
            <select
              className="session-filter-select"
              id="session-filter-project"
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  project: event.target.value
                }))
              }
              value={filters.project}
            >
              <option value="all">{copy.sessions.filters.options.allProjects}</option>
              {projectOptions.map((project) => (
                <option key={project} value={project}>
                  {project}
                </option>
              ))}
            </select>
          </label>
          <label className="session-filter-field" htmlFor="session-filter-risk">
            <span>{copy.sessions.filters.labels.risk}</span>
            <select
              className="session-filter-select"
              id="session-filter-risk"
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  risk: event.target.value as SessionFilterState["risk"]
                }))
              }
              value={filters.risk}
            >
              <option value="all">{copy.sessions.filters.options.allRisks}</option>
              <option value="at-risk">{copy.sessions.filters.options.atRisk}</option>
              <option value="clean">{copy.sessions.filters.options.clean}</option>
            </select>
          </label>
          <label className="session-filter-field" htmlFor="session-filter-export">
            <span>{copy.sessions.filters.labels.export}</span>
            <select
              className="session-filter-select"
              id="session-filter-export"
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  export: event.target.value as SessionFilterState["export"]
                }))
              }
              value={filters.export}
            >
              <option value="all">{copy.sessions.filters.options.allExports}</option>
              <option value="ready-to-quarantine">
                {copy.sessions.filters.options.readyToQuarantine}
              </option>
              <option value="needs-export">{copy.sessions.filters.options.needsExport}</option>
            </select>
          </label>
          <label className="session-filter-field" htmlFor="session-filter-control">
            <span>{copy.sessions.filters.labels.control}</span>
            <select
              className="session-filter-select"
              id="session-filter-control"
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  control: event.target.value as SessionFilterState["control"]
                }))
              }
              value={filters.control}
            >
              <option value="all">{copy.sessions.filters.options.allControls}</option>
              <option value="controllable">{copy.sessions.filters.options.controllable}</option>
              <option value="attached">{copy.sessions.filters.options.attached}</option>
            </select>
          </label>
        </div>
        {hasActiveFilters ? (
          <div className="session-filter-actions">
            <button
              className="action-button action-button-secondary"
              onClick={() => setFilters(DEFAULT_SESSION_FILTERS)}
              type="button"
            >
              {copy.sessions.filters.reset}
            </button>
          </div>
        ) : null}
        <p className="search-summary">
          {isSearching
            ? copy.sessions.searchSummaryPending
            : committedQuery
            ? `${filteredSessions.length} ${copy.sessions.searchSummary}`
            : hasActiveFilters
            ? `${filteredSessions.length} ${copy.sessions.filterSummary}`
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
          onAttachSession={onAttachSession}
          onContinueSession={onContinueSession}
          onDetachSession={onDetachSession}
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

function uniqueValues(values: string[]) {
  return [...new Set(values.filter((value) => value.trim().length > 0))].sort((left, right) =>
    left.localeCompare(right)
  );
}
