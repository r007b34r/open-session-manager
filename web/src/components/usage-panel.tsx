import type {
  CostSource,
  UsageOverviewRecord,
  UsageTimelineRecord
} from "../lib/api";
import { useI18n } from "../lib/i18n";

type UsagePanelProps = {
  usageOverview: UsageOverviewRecord;
  usageTimeline: UsageTimelineRecord[];
};

export function UsagePanel({ usageOverview, usageTimeline }: UsagePanelProps) {
  const { copy } = useI18n();
  const unknownValue = copy.data.unknownValue;
  const maxTimelineTokens = usageTimeline.reduce(
    (max, entry) => Math.max(max, entry.totalTokens),
    0
  );

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
              <strong>{formatUsd(usageOverview.totals.costUsd, unknownValue)}</strong>
              <small className="usage-cost-note">
                {formatCostSource(copy, usageOverview.totals.costSource)}
              </small>
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
                  <strong>{formatUsd(assistant.costUsd, unknownValue)}</strong>
                </div>
                <p className="usage-cost-note">
                  {formatCostSource(copy, assistant.costSource)}
                </p>
              </article>
            ))}
          </div>
        </section>

        <section className="usage-card usage-card-wide">
          <h3>{copy.overview.usageTimelineTitle}</h3>
          {usageTimeline.length === 0 ? (
            <p className="runtime-note">{copy.overview.usageTimelineEmpty}</p>
          ) : (
            <div className="usage-timeline-list">
              {usageTimeline.map((entry) => {
                const width =
                  maxTimelineTokens > 0
                    ? `${Math.max((entry.totalTokens / maxTimelineTokens) * 100, 8)}%`
                    : "8%";

                return (
                  <article className="usage-timeline-row" key={entry.date}>
                    <div className="usage-timeline-topline">
                      <strong>{entry.date}</strong>
                      <span className="badge badge-neutral">
                        {copy.overview.usageFields.sessionCount}: {entry.sessionsWithUsage}
                      </span>
                    </div>
                    <div className="usage-timeline-bar" aria-hidden="true">
                      <span style={{ width }} />
                    </div>
                    <div className="usage-inline">
                      <span>{copy.overview.usageFields.totalTokens}</span>
                      <strong>{formatCount(entry.totalTokens)}</strong>
                    </div>
                    <div className="usage-inline">
                      <span>{copy.overview.usageFields.totalCost}</span>
                      <strong>
                        {entry.costSource === "unknown"
                          ? copy.overview.costUnavailable
                          : formatUsd(entry.costUsd, unknownValue)}
                      </strong>
                    </div>
                    <p className="usage-cost-note">
                      {formatCostSource(copy, entry.costSource)}
                    </p>
                  </article>
                );
              })}
            </div>
          )}
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

function formatCostSource(
  copy: ReturnType<typeof useI18n>["copy"],
  source: CostSource
) {
  switch (source) {
    case "estimated":
      return copy.overview.costSources.estimated;
    case "mixed":
      return copy.overview.costSources.mixed;
    case "reported":
      return copy.overview.costSources.reported;
    default:
      return copy.overview.costSources.unknown;
  }
}
