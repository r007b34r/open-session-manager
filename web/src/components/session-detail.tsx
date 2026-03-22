import { useEffect, useState } from "react";

import type { SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type SessionDetailProps = {
  session?: SessionDetailRecord;
  canSoftDelete?: boolean;
  exportPath?: string;
  onContinueSession?: (sessionId: string, prompt: string) => void;
  onExportMarkdown?: (sessionId: string) => void;
  onResumeSession?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
  transcriptFocus?: {
    highlightIndex: number;
    terms: string[];
  };
};

export function SessionDetail({
  session,
  canSoftDelete = false,
  exportPath,
  onContinueSession,
  onExportMarkdown,
  onResumeSession,
  onSoftDelete,
  transcriptFocus
}: SessionDetailProps) {
  const { copy, translateProgressState, translateRiskFlag } = useI18n();
  const unknownValue = copy.data.unknownValue;
  const [continuePrompt, setContinuePrompt] = useState("");

  useEffect(() => {
    setContinuePrompt("");
  }, [session?.sessionId]);

  if (!session) {
    return (
      <section className="panel detail-panel">
        <p className="section-kicker">{copy.sessionDetail.kicker}</p>
        <h2>{copy.sessionDetail.emptyTitle}</h2>
        <p className="panel-copy">{copy.sessionDetail.emptyBody}</p>
      </section>
    );
  }

  return (
    <section className="panel detail-panel">
      <div className="detail-hero">
        <div className="detail-hero-copy">
          <p className="section-kicker">{copy.sessionDetail.kicker}</p>
          <h2>{session.title}</h2>
          <p className="detail-summary">{session.summary}</p>
          <div className="badge-row detail-topline">
            <span className="badge badge-neutral">{session.assistant}</span>
            <span className="badge badge-neutral">{session.environment}</span>
            <span className="badge badge-safe">
              {translateProgressState(session.progressState)}
            </span>
          </div>
        </div>

        <div className="detail-metric-strip">
          <div className="detail-metric-card">
            <span>{copy.sessionDetail.fields.completion}</span>
            <strong>{session.progressPercent}%</strong>
          </div>
          <div className="detail-metric-card">
            <span>{copy.sessionDetail.fields.valueScore}</span>
            <strong>{session.valueScore}</strong>
          </div>
          <div className="detail-metric-card">
            <span>{copy.sessionDetail.fields.lastActive}</span>
            <strong>{session.lastActivityAt}</strong>
          </div>
        </div>
      </div>

      <div className="action-row detail-action-row">
        <button
          className="action-button action-button-secondary"
          disabled={
            !onResumeSession ||
            !session.sessionControl?.supported ||
            !session.sessionControl?.available
          }
          onClick={() => onResumeSession?.(session.sessionId)}
          type="button"
        >
          {copy.sessionDetail.actions.resumeSession}
        </button>
        <button
          className="action-button action-button-primary"
          onClick={() => onExportMarkdown?.(session.sessionId)}
          type="button"
        >
          {copy.sessionDetail.actions.exportMarkdown}
        </button>
        <button
          className="action-button action-button-danger"
          disabled={!canSoftDelete}
          onClick={() => onSoftDelete?.(session.sessionId)}
          type="button"
        >
          {copy.sessionDetail.actions.moveToQuarantine}
        </button>
      </div>
      {session.sessionControl?.supported && !session.sessionControl.available ? (
        <p className="action-hint">{copy.sessionDetail.controlUnavailable}</p>
      ) : null}
      {!canSoftDelete ? (
        <p className="action-hint">
          {copy.sessionDetail.cleanupRequirement}
        </p>
      ) : null}
      {exportPath ? (
        <p className="action-success">
          {copy.sessionDetail.exportPathLabel}: <span>{exportPath}</span>
        </p>
      ) : null}

      <div className="detail-card-grid">
        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.sessionControl}</h3>
          {session.sessionControl ? (
            <div className="detail-control-stack">
              <ul className="detail-list">
                <li>
                  {copy.sessionDetail.fields.controller}: {session.sessionControl.controller}
                </li>
                <li>
                  {copy.sessionDetail.fields.command}:{" "}
                  {session.sessionControl.command || unknownValue}
                </li>
                <li>
                  {copy.sessionDetail.fields.controlStatus}:{" "}
                  {session.sessionControl.attached
                    ? copy.sessionDetail.statuses.attached
                    : copy.sessionDetail.statuses.detached}
                </li>
                <li>
                  {copy.sessionDetail.fields.lastResumeAt}:{" "}
                  {session.sessionControl.lastResumedAt ?? unknownValue}
                </li>
                <li>
                  {copy.sessionDetail.fields.lastContinueAt}:{" "}
                  {session.sessionControl.lastContinuedAt ?? unknownValue}
                </li>
              </ul>
              <label className="search-label" htmlFor="session-continue-prompt">
                {copy.sessionDetail.fields.continuePrompt}
              </label>
              <textarea
                className="detail-control-textarea"
                id="session-continue-prompt"
                onChange={(event) => setContinuePrompt(event.target.value)}
                placeholder={copy.sessionDetail.continuePlaceholder}
                value={continuePrompt}
              />
              <div className="action-row">
                <button
                  className="action-button action-button-secondary"
                  disabled={
                    !onContinueSession ||
                    !session.sessionControl.available ||
                    continuePrompt.trim().length === 0
                  }
                  onClick={() => {
                    const trimmed = continuePrompt.trim();
                    if (!trimmed) {
                      return;
                    }
                    onContinueSession?.(session.sessionId, trimmed);
                    setContinuePrompt("");
                  }}
                  type="button"
                >
                  {copy.sessionDetail.actions.continueSession}
                </button>
              </div>
              {session.sessionControl.lastPrompt ? (
                <p className="action-success">
                  {copy.sessionDetail.fields.lastPrompt}:{" "}
                  <span>{session.sessionControl.lastPrompt}</span>
                </p>
              ) : null}
              {session.sessionControl.lastResponse ? (
                <p className="action-success">
                  {copy.sessionDetail.fields.lastResponse}:{" "}
                  <span>{session.sessionControl.lastResponse}</span>
                </p>
              ) : null}
              {session.sessionControl.lastError ? (
                <p className="action-hint">
                  {copy.sessionDetail.fields.lastError}: {session.sessionControl.lastError}
                </p>
              ) : null}
            </div>
          ) : (
            <p className="detail-empty-copy">{copy.sessionDetail.noSessionControl}</p>
          )}
        </section>

        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.context}</h3>
          <ul className="detail-list">
            <li>
              {copy.sessionDetail.fields.assistant}: {session.assistant}
            </li>
            <li>
              {copy.sessionDetail.fields.environment}: {session.environment}
            </li>
            <li>{copy.sessionDetail.fields.project}: {session.projectPath}</li>
            <li>{copy.sessionDetail.fields.source}: {session.sourcePath}</li>
          </ul>
        </section>

        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.signals}</h3>
          <ul className="detail-list">
            <li>
              {copy.sessionDetail.fields.progress}:{" "}
              {translateProgressState(session.progressState)}
            </li>
            <li>
              {copy.sessionDetail.fields.completion}: {session.progressPercent}%
            </li>
            <li>
              {copy.sessionDetail.fields.valueScore}: {session.valueScore}
            </li>
            <li>
              {copy.sessionDetail.fields.lastActive}: {session.lastActivityAt}
            </li>
          </ul>
        </section>

        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.usage}</h3>
          {session.usage ? (
            <ul className="detail-list">
              <li>
                {copy.sessionDetail.fields.model}: {session.usage.model ?? unknownValue}
              </li>
              <li>
                {copy.sessionDetail.fields.inputTokens}: {formatCount(session.usage.inputTokens)}
              </li>
              <li>
                {copy.sessionDetail.fields.outputTokens}: {formatCount(session.usage.outputTokens)}
              </li>
              <li>
                {copy.sessionDetail.fields.cacheReadTokens}:{" "}
                {formatCount(session.usage.cacheReadTokens)}
              </li>
              <li>
                {copy.sessionDetail.fields.cacheWriteTokens}:{" "}
                {formatCount(session.usage.cacheWriteTokens)}
              </li>
              <li>
                {copy.sessionDetail.fields.reasoningTokens}:{" "}
                {formatCount(session.usage.reasoningTokens)}
              </li>
              <li>
                {copy.sessionDetail.fields.totalTokens}: {formatCount(session.usage.totalTokens)}
              </li>
              <li>
                {copy.sessionDetail.fields.costUsd}:{" "}
                {formatUsd(session.usage.costUsd, unknownValue)}
              </li>
            </ul>
          ) : (
            <p className="detail-empty-copy">
              {copy.sessionDetail.noTranscriptHighlights}
            </p>
          )}
        </section>

        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.keyArtifacts}</h3>
          <ul className="detail-list">
            {session.keyArtifacts.map((artifact) => (
              <li key={artifact}>{artifact}</li>
            ))}
          </ul>
        </section>

        <section className="detail-card detail-card--wide">
          <h3>{copy.sessionDetail.sections.transcriptHighlights}</h3>
          {session.transcriptHighlights.length === 0 ? (
            <p className="detail-empty-copy">
              {copy.sessionDetail.noTranscriptHighlights}
            </p>
          ) : (
            <div className="detail-transcript-list">
              {session.transcriptHighlights.map((highlight, index) => (
                <article
                  className={
                    transcriptFocus?.highlightIndex === index
                      ? "detail-transcript-entry is-search-match"
                      : "detail-transcript-entry"
                  }
                  key={`${highlight.role}-${index}-${highlight.content}`}
                >
                  <div className="badge-row">
                    <span className="badge badge-neutral">{highlight.role}</span>
                    {transcriptFocus?.highlightIndex === index ? (
                      <span className="badge badge-safe">
                        {copy.sessionDetail.statuses.searchHit}
                      </span>
                    ) : null}
                  </div>
                  <p>
                    {transcriptFocus?.highlightIndex === index
                      ? highlightSearchTerms(highlight.content, transcriptFocus.terms)
                      : highlight.content}
                  </p>
                </article>
              ))}
            </div>
          )}
        </section>

        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.todoSnapshot}</h3>
          {session.todoItems.length === 0 ? (
            <p className="detail-empty-copy">
              {copy.sessionDetail.noTodoItems}
            </p>
          ) : (
            <ul className="detail-list detail-todo-list">
              {session.todoItems.map((todo) => (
                <li
                  className={
                    todo.completed
                      ? "detail-todo-item is-completed"
                      : "detail-todo-item"
                  }
                  key={`${todo.content}-${todo.completed}`}
                >
                  <span className="detail-todo-check" aria-hidden="true">
                    {todo.completed ? "[x]" : "[ ]"}
                  </span>
                  <span>{todo.content}</span>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.riskFlags}</h3>
          <div className="badge-row">
            {session.riskFlags.length === 0 ? (
              <span className="badge badge-safe">
                {copy.sessionDetail.noRiskFlags}
              </span>
            ) : (
              session.riskFlags.map((flag) => (
                <span className="badge badge-risk" key={flag}>
                  {translateRiskFlag(flag)}
                </span>
              ))
            )}
          </div>
          <h3>{copy.sessionDetail.sections.topicLabels}</h3>
          <div className="badge-row">
            {session.tags.map((tag) => (
              <span className="badge badge-neutral" key={tag}>
                {tag}
              </span>
            ))}
          </div>
        </section>
      </div>
    </section>
  );
}

function formatCount(value: number) {
  return new Intl.NumberFormat("en-US").format(value);
}

function formatUsd(value: number | undefined, unknownValue: string) {
  if (typeof value !== "number") {
    return unknownValue;
  }

  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(value);
}

function highlightSearchTerms(text: string, terms: string[]) {
  const normalizedTerms = [...new Set(terms.map((term) => term.trim()).filter(Boolean))].sort(
    (left, right) => right.length - left.length
  );
  if (normalizedTerms.length === 0) {
    return text;
  }

  const matcher = new RegExp(
    `(${normalizedTerms.map((term) => escapeRegExp(term)).join("|")})`,
    "gi"
  );
  const segments = text.split(matcher);

  return segments.map((segment, index) => {
    if (!segment) {
      return null;
    }

    const isMatch = normalizedTerms.some((term) => term.toLowerCase() === segment.toLowerCase());
    if (!isMatch) {
      return <span key={`${segment}-${index}`}>{segment}</span>;
    }

    return <mark key={`${segment}-${index}`}>{segment}</mark>;
  });
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
