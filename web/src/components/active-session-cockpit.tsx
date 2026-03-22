import type { SessionDetailRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type ActiveSessionCockpitProps = {
  sessions: SessionDetailRecord[];
  isRefreshing: boolean;
  onRefresh: () => void;
};

export function ActiveSessionCockpit({
  sessions,
  isRefreshing,
  onRefresh
}: ActiveSessionCockpitProps) {
  const { copy } = useI18n();
  const controllableSessions = sessions.filter((session) => {
    const control = session.sessionControl;
    return Boolean(control && (control.supported || control.available || control.attached));
  });

  return (
    <section className="panel cockpit-panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.overview.cockpit.kicker}</p>
          <h2>{copy.overview.cockpit.title}</h2>
        </div>
        <div className="cockpit-actions">
          <p className="panel-copy">{copy.overview.cockpit.description}</p>
          <button
            className="action-button action-button-secondary"
            disabled={isRefreshing}
            onClick={onRefresh}
            type="button"
          >
            {isRefreshing
              ? copy.overview.cockpit.actions.refreshing
              : copy.overview.cockpit.actions.refresh}
          </button>
        </div>
      </div>

      {controllableSessions.length === 0 ? (
        <p className="panel-copy">{copy.overview.cockpit.empty}</p>
      ) : (
        <div className="cockpit-grid">
          {controllableSessions.map((session) => {
            const control = session.sessionControl;
            if (!control) {
              return null;
            }

            const lastTimestamp =
              control.lastContinuedAt ?? control.lastResumedAt ?? session.lastActivityAt;

            return (
              <article className="cockpit-card" key={session.sessionId}>
                <div className="config-card-topline">
                  <strong>{session.title}</strong>
                  <span className="badge badge-safe">
                    {translateCockpitStatus(copy.overview.cockpit.statuses, control)}
                  </span>
                </div>
                <div className="badge-row">
                  <span className="badge badge-neutral">{session.assistant}</span>
                  <span className="badge badge-neutral">{control.controller}</span>
                </div>
                <dl className="config-meta">
                  <div>
                    <dt>{copy.overview.cockpit.fields.command}</dt>
                    <dd>{control.command || copy.data.unknownValue}</dd>
                  </div>
                  <div>
                    <dt>{copy.overview.cockpit.fields.lastSeen}</dt>
                    <dd>{lastTimestamp}</dd>
                  </div>
                </dl>
                <div className="cockpit-copy-stack">
                  <p>
                    <strong>{copy.overview.cockpit.fields.lastResponse}</strong>
                    {": "}
                    {control.lastResponse ?? copy.overview.cockpit.noRecentResponse}
                  </p>
                  {control.lastError ? (
                    <p>
                      <strong>{copy.overview.cockpit.fields.lastError}</strong>
                      {": "}
                      {control.lastError}
                    </p>
                  ) : null}
                </div>
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}

function translateCockpitStatus(
  statuses: {
    attached: string;
    ready: string;
    unavailable: string;
  },
  control: NonNullable<SessionDetailRecord["sessionControl"]>
) {
  if (control.attached) {
    return statuses.attached;
  }

  if (control.available) {
    return statuses.ready;
  }

  return statuses.unavailable;
}
