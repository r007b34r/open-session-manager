import type { ReviewDiffEntry } from "../lib/review-flow";
import { useI18n } from "../lib/i18n";

type DiffViewerProps = {
  entries: ReviewDiffEntry[];
};

export function DiffViewer({ entries }: DiffViewerProps) {
  const { copy } = useI18n();

  return (
    <div className="review-diff-list">
      {entries.map((entry) => (
        <article className="review-diff-card" key={`${entry.field}-${entry.before}-${entry.after}`}>
          <div className="config-card-topline">
            <strong>{copy.configRisk.review.fieldLabels[entry.field]}</strong>
            <span
              className={
                entry.severity === "warning" ? "badge badge-risk" : "badge badge-safe"
              }
            >
              {copy.configRisk.review.severityLabels[entry.severity]}
            </span>
          </div>
          <dl className="review-diff-grid">
            <div>
              <dt>{copy.configRisk.review.beforeLabel}</dt>
              <dd>{entry.before}</dd>
            </div>
            <div>
              <dt>{copy.configRisk.review.afterLabel}</dt>
              <dd>{entry.after}</dd>
            </div>
          </dl>
        </article>
      ))}
    </div>
  );
}
