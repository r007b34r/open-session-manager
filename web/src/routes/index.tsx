import type { DashboardSnapshot } from "../lib/api";
import { useI18n } from "../lib/i18n";

type OverviewRouteProps = {
  snapshot: DashboardSnapshot;
};

export function OverviewRoute({ snapshot }: OverviewRouteProps) {
  const { translateMetricLabel, translateMetricNote } = useI18n();

  return (
    <section className="overview-grid">
      {snapshot.metrics.map((metric) => (
        <article className="metric-card" key={metric.label}>
          <p className="section-kicker">{translateMetricLabel(metric.label)}</p>
          <strong>{metric.value}</strong>
          <span>{translateMetricNote(metric.note)}</span>
        </article>
      ))}
    </section>
  );
}
