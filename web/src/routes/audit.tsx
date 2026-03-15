import type { AuditEventRecord } from "../lib/api";

type AuditRouteProps = {
  events: AuditEventRecord[];
};

export function AuditRoute({ events }: AuditRouteProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">Audit Center</p>
          <h2>Trace every destructive operation</h2>
        </div>
        <p className="panel-copy">
          Export, quarantine, and restore actions stay attached to an actor,
          timestamp, and target.
        </p>
      </div>

      <div className="audit-stack">
        {events.map((event) => (
          <article className="audit-card" key={event.eventId}>
            <div className="audit-card-head">
              <strong>{event.type}</strong>
              <span className="badge badge-neutral">{event.result}</span>
            </div>
            <p>{event.detail}</p>
            <div className="audit-meta">
              <span>{event.target}</span>
              <span>{event.actor}</span>
              <span>{event.createdAt}</span>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}
