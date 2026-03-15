import type { ConfigRiskRecord } from "../lib/api";

type ConfigRiskPanelProps = {
  configs: ConfigRiskRecord[];
};

export function ConfigRiskPanel({ configs }: ConfigRiskPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">Config Center</p>
          <h2>Config Risk Center</h2>
        </div>
        <p className="panel-copy">
          Secrets stay masked by default while endpoints, providers, and risk
          posture remain visible.
        </p>
      </div>

      <div className="config-grid">
        {configs.map((config) => (
          <article className="config-card" key={config.artifactId}>
            <div className="config-card-topline">
              <span className="badge badge-neutral">{config.assistant}</span>
              <span className="badge badge-risk">{config.officialOrProxy}</span>
            </div>
            <h3>{config.path}</h3>
            <dl className="config-meta">
              <div>
                <dt>Scope</dt>
                <dd>{config.scope}</dd>
              </div>
              <div>
                <dt>Provider</dt>
                <dd>{config.provider}</dd>
              </div>
              <div>
                <dt>Endpoint</dt>
                <dd>{config.baseUrl}</dd>
              </div>
              <div>
                <dt>Masked Key</dt>
                <dd>{config.maskedSecret}</dd>
              </div>
            </dl>
            <div className="badge-row">
              {config.risks.map((risk) => (
                <span className="badge badge-risk" key={risk}>
                  {risk}
                </span>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}
