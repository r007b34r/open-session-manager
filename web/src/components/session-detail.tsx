import type { SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type SessionDetailProps = {
  session?: SessionDetailRecord;
  onExportMarkdown?: (sessionId: string) => void;
  onSoftDelete?: (sessionId: string) => void;
};

export function SessionDetail({
  session,
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
      <p className="section-kicker">{copy.sessionDetail.kicker}</p>
      <h2>{session.title}</h2>
      <p className="detail-summary">{session.summary}</p>

      <div className="action-row">
        <button
          className="action-button action-button-primary"
          onClick={() => onExportMarkdown?.(session.sessionId)}
          type="button"
        >
          {copy.sessionDetail.actions.exportMarkdown}
        </button>
        <button
          className="action-button action-button-danger"
          onClick={() => onSoftDelete?.(session.sessionId)}
          type="button"
        >
          {copy.sessionDetail.actions.moveToQuarantine}
        </button>
      </div>

      <div className="detail-grid">
        <div>
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
        </div>

        <div>
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
        </div>
      </div>

      <div className="detail-grid">
        <div>
          <h3>{copy.sessionDetail.sections.keyArtifacts}</h3>
          <ul className="detail-list">
            {session.keyArtifacts.map((artifact) => (
              <li key={artifact}>{artifact}</li>
            ))}
          </ul>
        </div>

        <div>
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
        </div>
      </div>
    </section>
  );
}
