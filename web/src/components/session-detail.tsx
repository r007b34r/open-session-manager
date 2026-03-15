import type { SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type SessionDetailProps = {
  session?: SessionDetailRecord;
  canSoftDelete?: boolean;
  onExportMarkdown?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionDetail({
  session,
  canSoftDelete = false,
  onExportMarkdown,
  onSoftDelete
}: SessionDetailProps) {
  const { copy, translateProgressState, translateRiskFlag } = useI18n();

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
      {!canSoftDelete ? (
        <p className="action-hint">
          {copy.sessionDetail.cleanupRequirement}
        </p>
      ) : null}

      <div className="detail-card-grid">
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
                  className="detail-transcript-entry"
                  key={`${highlight.role}-${index}-${highlight.content}`}
                >
                  <span className="badge badge-neutral">{highlight.role}</span>
                  <p>{highlight.content}</p>
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
