import type { DashboardSnapshot } from "../lib/api";

type OverviewRouteProps = {
  snapshot: DashboardSnapshot;
};

export function OverviewRoute({ snapshot }: OverviewRouteProps) {
  return (
    <section className="overview-grid">
      {snapshot.metrics.map((metric) => (
        <article className="metric-card" key={metric.label}>
          <p className="section-kicker">{metric.label}</p>
          <strong>{metric.value}</strong>
          <span>{metric.note}</span>
        </article>
      ))}
    </section>
  );
}
