import { DoctorPanel } from "../components/doctor-panel";
import type { DashboardSnapshot } from "../lib/api";
import { UsagePanel } from "../components/usage-panel";
import { useI18n } from "../lib/i18n";

type OverviewRouteProps = {
  snapshot: DashboardSnapshot;
};

export function OverviewRoute({ snapshot }: OverviewRouteProps) {
  const { copy, language, translateMetricLabel, translateMetricNote } = useI18n();
  const adoptedUpstreams =
    language === "zh-CN"
      ? [
          {
            repository: "daaain/claude-code-log",
            badge: copy.overview.adoptedBadge,
            summary: "更丰富的 Markdown 分节、摘要导出和 transcript highlights 已落进当前导出链路。"
          },
          {
            repository: "d-kimuson/claude-code-viewer",
            badge: copy.overview.adoptedBadge,
            summary: "viewer 风格 transcript detail、todo evidence 和更贴近真实会话的详情面板已在当前界面可见。"
          }
        ]
      : [
          {
            repository: "daaain/claude-code-log",
            badge: copy.overview.adoptedBadge,
            summary:
              "Markdown sections, transcript highlights, and cleanup-first exports were absorbed into the current export flow."
          },
          {
            repository: "d-kimuson/claude-code-viewer",
            badge: copy.overview.adoptedBadge,
            summary:
              "Viewer-style transcript detail, todo evidence, and a more faithful session detail panel are visible in the current UI."
          }
        ];
  const researchedUpstreams =
    language === "zh-CN"
      ? [
          {
            repository: "lulu-sk/CodexFlow",
            badge: copy.overview.researchBadge,
            summary: "继续参考其多助手工作区和 Windows / WSL 并行整理方式。"
          },
          {
            repository: "jazzyalex/agent-sessions",
            badge: copy.overview.researchBadge,
            summary: "继续吸收本地索引、搜索和 analytics 方向的组织方式。"
          }
        ]
      : [
          {
            repository: "lulu-sk/CodexFlow",
            badge: copy.overview.researchBadge,
            summary:
              "Still informing the multi-assistant workspace and Windows plus WSL queue design."
          },
          {
            repository: "jazzyalex/agent-sessions",
            badge: copy.overview.researchBadge,
            summary:
              "Still guiding the local indexing, search, and analytics roadmap."
          }
        ];

  return (
    <>
      <section className="overview-grid">
        {snapshot.metrics.map((metric) => (
          <article className="metric-card" key={metric.label}>
            <p className="section-kicker">{translateMetricLabel(metric.label)}</p>
            <strong>{metric.value}</strong>
            <span>{translateMetricNote(metric.note)}</span>
          </article>
        ))}
      </section>

      <DoctorPanel findings={snapshot.doctorFindings} />

      <UsagePanel usageOverview={snapshot.usageOverview} />

      <section className="panel adoption-panel">
        <div className="panel-header">
          <div>
            <p className="section-kicker">{copy.overview.adoptionKicker}</p>
            <h2>{copy.overview.adoptionTitle}</h2>
          </div>
          <p className="panel-copy">{copy.overview.adoptionDescription}</p>
        </div>

        <div className="adoption-grid">
          <section className="adoption-column">
            <h3>{copy.overview.adoptedTitle}</h3>
            <div className="adoption-stack">
              {adoptedUpstreams.map((item) => (
                <article className="adoption-card" key={item.repository}>
                  <div className="config-card-topline">
                    <strong>{item.repository}</strong>
                    <span className="badge badge-safe">{item.badge}</span>
                  </div>
                  <p>{item.summary}</p>
                </article>
              ))}
            </div>
          </section>

          <section className="adoption-column">
            <h3>{copy.overview.researchTitle}</h3>
            <div className="adoption-stack">
              {researchedUpstreams.map((item) => (
                <article className="adoption-card" key={item.repository}>
                  <div className="config-card-topline">
                    <strong>{item.repository}</strong>
                    <span className="badge badge-neutral">{item.badge}</span>
                  </div>
                  <p>{item.summary}</p>
                </article>
              ))}
            </div>
          </section>
        </div>
      </section>
    </>
  );
}
