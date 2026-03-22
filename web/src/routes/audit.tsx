import type { AuditEventRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type AuditRouteProps = {
  events: AuditEventRecord[];
};

export function AuditRoute({ events }: AuditRouteProps) {
  const { copy, translateAuditResult, translateAuditType } = useI18n();

  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.audit.kicker}</p>
          <h2>{copy.audit.title}</h2>
        </div>
        <p className="panel-copy">{copy.audit.description}</p>
      </div>

      <div className="audit-stack">
        {events.map((event) => (
          <article className="audit-card" key={event.eventId}>
            <div className="audit-card-head">
              <strong>{translateAuditType(event.type)}</strong>
              <span className="badge badge-neutral">
                {translateAuditResult(event.result)}
              </span>
            </div>
            <p>{event.detail}</p>
            <div className="audit-meta">
              <span>{event.target}</span>
              <span>{event.actor}</span>
              <span>{event.createdAt}</span>
            </div>
            {event.outputPath || event.quarantinedPath || event.manifestPath ? (
              <dl className="audit-paths">
                {event.outputPath ? (
                  <div>
                    <dt>{copy.audit.pathLabels.outputPath}</dt>
                    <dd>{event.outputPath}</dd>
                  </div>
                ) : null}
                {event.quarantinedPath ? (
                  <div>
                    <dt>{copy.audit.pathLabels.quarantinedPath}</dt>
                    <dd>{event.quarantinedPath}</dd>
                  </div>
                ) : null}
                {event.manifestPath ? (
                  <div>
                    <dt>{copy.audit.pathLabels.manifestPath}</dt>
                    <dd>{event.manifestPath}</dd>
                  </div>
                ) : null}
              </dl>
            ) : null}
          </article>
        ))}
      </div>
    </section>
  );
}
