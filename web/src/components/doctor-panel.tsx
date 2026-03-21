import type { DoctorFindingRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type DoctorPanelProps = {
  findings: DoctorFindingRecord[];
};

export function DoctorPanel({ findings }: DoctorPanelProps) {
  const { copy } = useI18n();

  return (
    <section className="panel doctor-panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.overview.doctorKicker}</p>
          <h2>{copy.overview.doctorTitle}</h2>
        </div>
        <p className="panel-copy">{copy.overview.doctorDescription}</p>
      </div>

      {findings.length === 0 ? (
        <p className="panel-copy">{copy.overview.doctorEmpty}</p>
      ) : (
        <div className="doctor-stack">
          {findings.map((finding) => (
            <article className="doctor-card" key={`${finding.code}-${finding.path}`}>
              <div className="config-card-topline">
                <strong>{finding.path}</strong>
                <span className="badge badge-risk">{finding.severity}</span>
              </div>
              <p className="panel-copy">{finding.detail}</p>
              <div className="badge-row">
                <span className="badge badge-neutral">{finding.assistant}</span>
                <span className="badge badge-neutral">{finding.code}</span>
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
