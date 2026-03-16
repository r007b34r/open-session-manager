import type { UsageOverviewRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type UsagePanelProps = {
  usageOverview: UsageOverviewRecord;
};

export function UsagePanel({ usageOverview }: UsagePanelProps) {
  const { copy } = useI18n();

  return (
    <section className="panel usage-panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.overview.usageKicker}</p>
          <h2>{copy.overview.usageTitle}</h2>
        </div>
        <p className="panel-copy">{copy.overview.usageDescription}</p>
      </div>

      <div className="usage-grid">
        <section className="usage-card">
          <h3>{copy.overview.usageTotalsTitle}</h3>
          <div className="usage-total-grid">
            <article className="usage-total-stat">
              <span>{copy.overview.usageFields.sessionsWithUsage}</span>
              <strong>{formatCount(usageOverview.totals.sessionsWithUsage)}</strong>
            </article>
            <article className="usage-total-stat">
              <span>{copy.overview.usageFields.totalTokens}</span>
              <strong>{formatCount(usageOverview.totals.totalTokens)}</strong>
            </article>
            <article className="usage-total-stat">
              <span>{copy.overview.usageFields.totalCost}</span>
              <strong>{formatUsd(usageOverview.totals.costUsd)}</strong>
            </article>
            <article className="usage-total-stat">
              <span>{copy.overview.usageFields.cacheRead}</span>
              <strong>{formatCount(usageOverview.totals.cacheReadTokens)}</strong>
            </article>
          </div>
        </section>

        <section className="usage-card">
          <h3>{copy.overview.usageAssistantsTitle}</h3>
          <div className="usage-assistant-list">
            {usageOverview.assistants.map((assistant) => (
              <article className="usage-assistant-card" key={assistant.assistant}>
                <div className="config-card-topline">
                  <strong>{assistant.assistant}</strong>
                  <span className="badge badge-neutral">
                    {copy.overview.usageFields.sessionCount}: {assistant.sessionCount}
                  </span>
                </div>
                <div className="usage-inline">
                  <span>{copy.overview.usageFields.totalTokens}</span>
                  <strong>{formatCount(assistant.totalTokens)}</strong>
                </div>
                <div className="usage-inline">
                  <span>{copy.overview.usageFields.totalCost}</span>
                  <strong>{formatUsd(assistant.costUsd)}</strong>
                </div>
              </article>
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

function formatUsd(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(value);
}
