import { useEffect, useState } from "react";

import { getSessionContinueGuard, type SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";
import { buildRuleArtifact, buildSkillArtifact } from "../lib/knowledge-lift";

type SessionDetailProps = {
  session?: SessionDetailRecord;
  canSoftDelete?: boolean;
  exportPath?: string;
  onAttachSession?: (sessionId: string) => void;
  onContinueSession?: (sessionId: string, prompt: string) => void;
  onDetachSession?: (sessionId: string) => void;
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
  onAttachSession,
  onContinueSession,
  onDetachSession,
  onExportMarkdown,
  onResumeSession,
  onSoftDelete,
  transcriptFocus
}: SessionDetailProps) {
  const { copy, translateProgressState, translateRiskFlag } = useI18n();
  const unknownValue = copy.data.unknownValue;
  const [continuePrompt, setContinuePrompt] = useState("");
  const [showCleanupReview, setShowCleanupReview] = useState(false);
  const [cleanupConfirmed, setCleanupConfirmed] = useState(false);
  const [liftView, setLiftView] = useState<"rule" | "skill">("rule");

  useEffect(() => {
    setContinuePrompt("");
    setShowCleanupReview(false);
    setCleanupConfirmed(false);
    setLiftView("rule");
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

  const ruleArtifact = buildRuleArtifact(session);
  const skillArtifact = buildSkillArtifact(session);
  const activeArtifact = liftView === "rule" ? ruleArtifact : skillArtifact;
  const sessionControl = session.sessionControl;
  const runtimeStatus = translateSessionControlStatus(copy.sessionDetail.statuses, sessionControl);
  const continueGuard = getSessionContinueGuard(session);
  const canContinue =
    Boolean(onContinueSession) &&
    continueGuard === "ok" &&
    continuePrompt.trim().length > 0;

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
          onClick={() => {
            setShowCleanupReview(true);
            setCleanupConfirmed(false);
          }}
          type="button"
        >
          {copy.sessionDetail.actions.moveToQuarantine}
        </button>
      </div>
      {sessionControl?.supported && !sessionControl.available ? (
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
      {showCleanupReview ? (
        <section className="detail-card detail-review-card">
          <h3>{copy.sessionDetail.cleanupReview.title}</h3>
          <p className="panel-copy">{copy.sessionDetail.cleanupReview.description}</p>
          {session.riskFlags.length > 0 ? (
            <div className="badge-row">
              {session.riskFlags.map((flag) => (
                <span className="badge badge-risk" key={`cleanup-${flag}`}>
                  {translateRiskFlag(flag)}
                </span>
              ))}
            </div>
          ) : null}
          <label className="review-confirm">
            <input
              checked={cleanupConfirmed}
              onChange={(event) => setCleanupConfirmed(event.target.checked)}
              type="checkbox"
            />
            <span>{copy.sessionDetail.cleanupReview.confirmLabel}</span>
          </label>
          <div className="action-row">
            <button
              className="action-button action-button-danger"
              disabled={!cleanupConfirmed}
              onClick={() => onSoftDelete?.(session.sessionId)}
              type="button"
            >
              {copy.sessionDetail.cleanupReview.actions.confirm}
            </button>
            <button
              className="action-button"
              onClick={() => {
                setShowCleanupReview(false);
                setCleanupConfirmed(false);
              }}
              type="button"
            >
              {copy.sessionDetail.cleanupReview.actions.back}
            </button>
          </div>
        </section>
      ) : null}

      <div className="detail-card-grid">
        <section className="detail-card">
          <h3>{copy.sessionDetail.sections.sessionControl}</h3>
          {sessionControl ? (
            <div className="detail-control-stack">
              <ul className="detail-list">
                <li>
                  {copy.sessionDetail.fields.controller}: {sessionControl.controller}
                </li>
                <li>
                  {copy.sessionDetail.fields.command}:{" "}
                  {sessionControl.command || unknownValue}
                </li>
                <li>
                  {copy.sessionDetail.fields.controlStatus}: {runtimeStatus}
                </li>
                <li>
                  {copy.sessionDetail.fields.lastResumeAt}:{" "}
                  {sessionControl.lastResumedAt ?? unknownValue}
                </li>
                <li>
                  {copy.sessionDetail.fields.lastContinueAt}:{" "}
                  {sessionControl.lastContinuedAt ?? unknownValue}
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
                {!sessionControl.attached ? (
                  <button
                    className="action-button action-button-secondary"
                    disabled={!onAttachSession || !sessionControl.available}
                    onClick={() => onAttachSession?.(session.sessionId)}
                    type="button"
                  >
                    {copy.sessionDetail.actions.attachSession}
                  </button>
                ) : null}
                {sessionControl.attached ? (
                  <button
                    className="action-button"
                    disabled={!onDetachSession || !sessionControl.available}
                    onClick={() => onDetachSession?.(session.sessionId)}
                    type="button"
                  >
                    {copy.sessionDetail.actions.detachSession}
                  </button>
                ) : null}
                <button
                  className="action-button action-button-secondary"
                  disabled={!canContinue}
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
              {sessionControl.lastPrompt ? (
                <p className="action-success">
                  {copy.sessionDetail.fields.lastPrompt}:{" "}
                  <span>{sessionControl.lastPrompt}</span>
                </p>
              ) : null}
              {sessionControl.lastResponse ? (
                <p className="action-success">
                  {copy.sessionDetail.fields.lastResponse}:{" "}
                  <span>{sessionControl.lastResponse}</span>
                </p>
              ) : null}
              {sessionControl.lastError ? (
                <p className="action-hint">
                  {copy.sessionDetail.fields.lastError}: {sessionControl.lastError}
                </p>
              ) : null}
              {continueGuard === "busy" ? (
                <p className="action-hint">
                  {copy.sessionDetail.continueGuardHints.busy}
                </p>
              ) : null}
              {continueGuard === "throttled" ? (
                <p className="action-hint">
                  {copy.sessionDetail.continueGuardHints.throttled}
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
          <h3>{copy.sessionDetail.sections.knowledgeLift}</h3>
          <p className="detail-empty-copy">
            {copy.sessionDetail.knowledgeLift.description}
          </p>
          <div className="detail-artifact-toolbar" role="tablist">
            <button
              aria-selected={liftView === "rule"}
              className={
                liftView === "rule"
                  ? "detail-artifact-button is-active"
                  : "detail-artifact-button"
              }
              onClick={() => setLiftView("rule")}
              type="button"
            >
              {copy.sessionDetail.knowledgeLift.views.rule}
            </button>
            <button
              aria-selected={liftView === "skill"}
              className={
                liftView === "skill"
                  ? "detail-artifact-button is-active"
                  : "detail-artifact-button"
              }
              onClick={() => setLiftView("skill")}
              type="button"
            >
              {copy.sessionDetail.knowledgeLift.views.skill}
            </button>
          </div>
          <textarea
            className="detail-artifact-preview"
            readOnly
            aria-label={copy.sessionDetail.knowledgeLift.previewLabel}
            value={activeArtifact}
          />
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

function translateSessionControlStatus(
  statuses: {
    attached: string;
    detached: string;
    busy: string;
    waiting: string;
    idle: string;
    unavailable: string;
    searchHit: string;
  },
  control?: SessionDetailRecord["sessionControl"]
) {
  if (!control) {
    return statuses.unavailable;
  }

  switch (control.runtimeState) {
    case "busy":
      return statuses.busy;
    case "waiting":
      return statuses.waiting;
    case "idle":
      return statuses.idle;
    case "detached":
      return statuses.detached;
    case "unavailable":
      return statuses.unavailable;
    default:
      break;
  }

  if (!control.available) {
    return statuses.unavailable;
  }

  return control.attached ? statuses.attached : statuses.detached;
}
