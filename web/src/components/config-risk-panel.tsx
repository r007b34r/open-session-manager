import { useState } from "react";

import type { ConfigRiskRecord, ConfigWritebackInput } from "../lib/api";
import { useI18n } from "../lib/i18n";

type ConfigRiskPanelProps = {
  configs: ConfigRiskRecord[];
  canEditConfigs?: boolean;
  onSaveConfig?: (input: ConfigWritebackInput) => void;
};

export function ConfigRiskPanel({
  configs,
  canEditConfigs = false,
  onSaveConfig
}: ConfigRiskPanelProps) {
  const { copy, translateProxyMode, translateRiskFlag, translateScope } =
    useI18n();
  const [draft, setDraft] = useState<ConfigWritebackInput | null>(null);

  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.configRisk.kicker}</p>
          <h2>{copy.configRisk.title}</h2>
        </div>
        <p className="panel-copy">{copy.configRisk.description}</p>
      </div>

      <div className="config-grid">
        {configs.map((config) => (
          <article className="config-card" key={config.artifactId}>
            <div className="config-card-topline">
              <span className="badge badge-neutral">{config.assistant}</span>
              <span className="badge badge-risk">
                {translateProxyMode(config.officialOrProxy)}
              </span>
            </div>
            <h3>{config.path}</h3>
            <dl className="config-meta">
              <div>
                <dt>{copy.configRisk.fields.scope}</dt>
                <dd>{translateScope(config.scope)}</dd>
              </div>
              <div>
                <dt>{copy.configRisk.fields.provider}</dt>
                <dd>{config.provider}</dd>
              </div>
              {config.model ? (
                <div>
                  <dt>{copy.configRisk.fields.model}</dt>
                  <dd>{config.model}</dd>
                </div>
              ) : null}
              <div>
                <dt>{copy.configRisk.fields.endpoint}</dt>
                <dd>{config.baseUrl}</dd>
              </div>
              <div>
                <dt>{copy.configRisk.fields.maskedKey}</dt>
                <dd>{config.maskedSecret}</dd>
              </div>
            </dl>
            {canEditConfigs && isEditableAssistant(config.assistant) ? (
              <div className="action-row">
                {draft?.artifactId === config.artifactId ? (
                  <>
                    <button
                      className="action-button action-button-primary"
                      onClick={() => {
                        if (!draft) {
                          return;
                        }

                        onSaveConfig?.({
                          ...draft,
                          model: normalizeOptionalValue(draft.model),
                          secret: normalizeOptionalValue(draft.secret)
                        });
                        setDraft(null);
                      }}
                      type="button"
                    >
                      {copy.configRisk.actions.saveConfig}
                    </button>
                    <button
                      className="action-button"
                      onClick={() => setDraft(null)}
                      type="button"
                    >
                      {copy.configRisk.actions.cancelEdit}
                    </button>
                  </>
                ) : (
                  <button
                    className="action-button action-button-primary"
                    onClick={() =>
                      setDraft({
                        artifactId: config.artifactId,
                        assistant: config.assistant,
                        scope: config.scope,
                        path: config.path,
                        provider: config.provider,
                        model: config.model,
                        baseUrl: config.baseUrl,
                        secret: ""
                      })
                    }
                    type="button"
                  >
                    {copy.configRisk.actions.editConfig}
                  </button>
                )}
              </div>
            ) : null}
            {draft?.artifactId === config.artifactId ? (
              <form
                className="config-meta"
                onSubmit={(event) => {
                  event.preventDefault();
                  onSaveConfig?.({
                    ...draft,
                    model: normalizeOptionalValue(draft.model),
                    secret: normalizeOptionalValue(draft.secret)
                  });
                  setDraft(null);
                }}
              >
                <div>
                  <label>
                    {copy.configRisk.fields.provider}
                    <input
                      disabled={!isProviderEditable(config.assistant)}
                      onChange={(event) =>
                        setDraft((current) =>
                          current
                            ? { ...current, provider: event.target.value }
                            : current
                        )
                      }
                      type="text"
                      value={draft.provider}
                    />
                  </label>
                </div>
                <div>
                  <label>
                    {copy.configRisk.fields.model}
                    <input
                      onChange={(event) =>
                        setDraft((current) =>
                          current ? { ...current, model: event.target.value } : current
                        )
                      }
                      type="text"
                      value={draft.model ?? ""}
                    />
                  </label>
                </div>
                <div>
                  <label>
                    {copy.configRisk.fields.endpoint}
                    <input
                      onChange={(event) =>
                        setDraft((current) =>
                          current
                            ? { ...current, baseUrl: event.target.value }
                            : current
                        )
                      }
                      type="text"
                      value={draft.baseUrl}
                    />
                  </label>
                </div>
                <div>
                  <label>
                    {copy.configRisk.fields.newKey}
                    <input
                      onChange={(event) =>
                        setDraft((current) =>
                          current ? { ...current, secret: event.target.value } : current
                        )
                      }
                      type="password"
                      value={draft.secret ?? ""}
                    />
                  </label>
                </div>
              </form>
            ) : null}
            <div className="badge-row">
              {config.risks.map((risk) => (
                <span className="badge badge-risk" key={risk}>
                  {translateRiskFlag(risk)}
                </span>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function isEditableAssistant(assistant: string) {
  return [
    "github-copilot-cli",
    "github copilot cli",
    "factory-droid",
    "factory droid",
    "gemini-cli",
    "gemini cli",
    "openclaw",
    "open claw"
  ].includes(assistant.trim().toLowerCase());
}

function isProviderEditable(assistant: string) {
  return ["factory-droid", "factory droid"].includes(
    assistant.trim().toLowerCase()
  );
}

function normalizeOptionalValue(value: string | undefined) {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
}
