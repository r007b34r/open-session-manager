import { useState, type SetStateAction } from "react";

import type {
  ConfigRiskRecord,
  ConfigWritebackInput,
  LocalAuditEventInput
} from "../lib/api";
import {
  applyConfigSnippetToDraft,
  buildConfigSnippet,
  loadSavedConfigSnippets,
  parseConfigSnippet,
  saveConfigSnippetRecord,
  serializeConfigSnippet,
  type StoredConfigSnippetRecord
} from "../lib/config-snippets";
import { useI18n } from "../lib/i18n";
import {
  applyProviderPreset,
  buildConfigDraft,
  getProviderPresets
} from "../lib/provider-presets";
import { buildConfigReview, type ConfigReview } from "../lib/review-flow";
import { DiffViewer } from "./diff-viewer";

type ConfigRiskPanelProps = {
  configs: ConfigRiskRecord[];
  canEditConfigs?: boolean;
  onSaveConfig?: (input: ConfigWritebackInput) => void;
  onAuditEvent?: (input: LocalAuditEventInput) => void;
};

export function ConfigRiskPanel({
  configs,
  canEditConfigs = false,
  onSaveConfig,
  onAuditEvent
}: ConfigRiskPanelProps) {
  const { copy, translateProxyMode, translateRiskFlag, translateScope } =
    useI18n();
  const [draft, setDraft] = useState<ConfigWritebackInput | null>(null);
  const [savedSnippets, setSavedSnippets] = useState<StoredConfigSnippetRecord[]>(() =>
    loadSavedConfigSnippets()
  );
  const [snippetName, setSnippetName] = useState("");
  const [snippetExport, setSnippetExport] = useState("");
  const [snippetImport, setSnippetImport] = useState("");
  const [snippetError, setSnippetError] = useState<string | null>(null);
  const [configReview, setConfigReview] = useState<ConfigReview | null>(null);
  const [reviewConfirmed, setReviewConfirmed] = useState(false);

  const beginEditing = (config: ConfigRiskRecord) => {
    setDraft(buildConfigDraft(config));
    setConfigReview(null);
    setReviewConfirmed(false);
    setSnippetName("");
    setSnippetExport("");
    setSnippetImport("");
    setSnippetError(null);
  };

  const resetSnippetComposer = () => {
    setSnippetName("");
    setSnippetExport("");
    setSnippetImport("");
    setSnippetError(null);
  };

  const recordSnippetAudit = (input: LocalAuditEventInput) => {
    onAuditEvent?.(input);
  };

  const updateDraft = (nextValue: SetStateAction<ConfigWritebackInput | null>) => {
    setConfigReview(null);
    setReviewConfirmed(false);
    setDraft(nextValue);
  };

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
                      disabled={Boolean(configReview) && !reviewConfirmed}
                      onClick={() => {
                        if (!draft) {
                          return;
                        }

                        if (!configReview) {
                          setConfigReview(buildConfigReview(config, draft));
                          setReviewConfirmed(false);
                          return;
                        }

                        onSaveConfig?.({
                          ...draft,
                          model: normalizeOptionalValue(draft.model),
                          secret: normalizeOptionalValue(draft.secret)
                        });
                        setDraft(null);
                        setConfigReview(null);
                        setReviewConfirmed(false);
                      }}
                      type="button"
                    >
                      {configReview
                        ? copy.configRisk.review.actions.apply
                        : copy.configRisk.review.actions.review}
                    </button>
                    <button
                      className="action-button"
                      onClick={() => {
                        if (configReview) {
                          setConfigReview(null);
                          setReviewConfirmed(false);
                          return;
                        }

                        setDraft(null);
                        setConfigReview(null);
                        setReviewConfirmed(false);
                        resetSnippetComposer();
                      }}
                      type="button"
                    >
                      {configReview
                        ? copy.configRisk.review.actions.back
                        : copy.configRisk.actions.cancelEdit}
                    </button>
                  </>
                ) : (
                  <button
                    className="action-button action-button-primary"
                    onClick={() => beginEditing(config)}
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
                  if (!draft) {
                    return;
                  }

                  setConfigReview(buildConfigReview(config, draft));
                  setReviewConfirmed(false);
                }}
              >
                <div>
                  <label>
                    {copy.configRisk.fields.provider}
                    <input
                      disabled={!isProviderEditable(config.assistant)}
                      onChange={(event) =>
                        updateDraft((current) =>
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
                        updateDraft((current) =>
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
                        updateDraft((current) =>
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
                        updateDraft((current) =>
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
            {draft?.artifactId === config.artifactId && configReview ? (
              <section className="config-review-panel">
                <div className="config-preset-head">
                  <h4>{copy.configRisk.review.title}</h4>
                  <p>{copy.configRisk.review.description}</p>
                </div>
                <DiffViewer entries={configReview.entries} />
                {configReview.warnings.length > 0 ? (
                  <div className="review-warning-stack">
                    <strong>{copy.configRisk.review.warningsTitle}</strong>
                    <ul className="detail-list">
                      {configReview.warnings.map((warning) => (
                        <li key={warning}>
                          {copy.configRisk.review.warningMessages[warning]}
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : null}
                <label className="review-confirm">
                  <input
                    checked={reviewConfirmed}
                    onChange={(event) => setReviewConfirmed(event.target.checked)}
                    type="checkbox"
                  />
                  <span>{copy.configRisk.review.confirmLabel}</span>
                </label>
              </section>
            ) : null}
            {draft?.artifactId === config.artifactId ? (
              <section className="config-preset-panel">
                <div className="config-preset-head">
                  <strong>{copy.configRisk.presets.title}</strong>
                  <p>{copy.configRisk.presets.description}</p>
                </div>
                <div className="config-preset-list">
                  {getProviderPresets(config.assistant).map((preset) => (
                    <button
                      className="action-button action-button-secondary config-preset-chip"
                      key={preset.id}
                      onClick={() =>
                        updateDraft((current) =>
                          current ? applyProviderPreset(current, preset) : current
                        )
                      }
                      type="button"
                    >
                      {copy.configRisk.presets.options[preset.id]}
                    </button>
                  ))}
                  <button
                    className="action-button action-button-secondary config-preset-chip"
                    onClick={() => updateDraft(buildConfigDraft(config))}
                    type="button"
                  >
                    {copy.configRisk.presets.restoreDetectedValues}
                  </button>
                </div>
              </section>
            ) : null}
            {draft?.artifactId === config.artifactId ? (
              <section className="config-snippet-panel">
                <div className="config-preset-head">
                  <strong>{copy.configRisk.snippets.title}</strong>
                  <p>{copy.configRisk.snippets.description}</p>
                </div>
                <div className="config-meta">
                  <div>
                    <label>
                      {copy.configRisk.snippets.snippetName}
                      <input
                        onChange={(event) => setSnippetName(event.target.value)}
                        type="text"
                        value={snippetName}
                      />
                    </label>
                  </div>
                  <div>
                    <label>
                      {copy.configRisk.snippets.exportJson}
                      <textarea
                        className="config-snippet-textarea"
                        readOnly
                        value={snippetExport}
                      />
                    </label>
                  </div>
                  <div>
                    <label>
                      {copy.configRisk.snippets.importJson}
                      <textarea
                        className="config-snippet-textarea"
                        onChange={(event) => setSnippetImport(event.target.value)}
                        value={snippetImport}
                      />
                    </label>
                  </div>
                </div>
                {snippetError ? <p className="action-hint">{snippetError}</p> : null}
                <div className="action-row">
                  <button
                    className="action-button action-button-secondary"
                    onClick={() => {
                      if (!draft) {
                        return;
                      }

                      const snippet = buildConfigSnippet(draft, { name: snippetName });
                      const nextSnippets = saveConfigSnippetRecord(snippet);
                      setSavedSnippets(nextSnippets);
                      setSnippetName(snippet.name);
                      setSnippetExport(serializeConfigSnippet(snippet));
                      setSnippetError(null);
                      recordSnippetAudit({
                        type: "config_snippet_save",
                        target: draft.artifactId,
                        detail: `Saved config snippet ${snippet.name}.`
                      });
                    }}
                    type="button"
                  >
                    {copy.configRisk.snippets.actions.saveSnippet}
                  </button>
                  <button
                    className="action-button action-button-secondary"
                    onClick={() => {
                      if (!draft) {
                        return;
                      }

                      const snippet = buildConfigSnippet(draft, { name: snippetName });
                      setSnippetName(snippet.name);
                      setSnippetExport(serializeConfigSnippet(snippet));
                      setSnippetError(null);
                      recordSnippetAudit({
                        type: "config_snippet_export",
                        target: draft.artifactId,
                        detail: `Prepared config snippet export ${snippet.name}.`
                      });
                    }}
                    type="button"
                  >
                    {copy.configRisk.snippets.actions.prepareExport}
                  </button>
                  <button
                    className="action-button action-button-secondary"
                    onClick={() => {
                      if (!draft) {
                        return;
                      }

                      try {
                        const snippet = parseConfigSnippet(snippetImport);
                        updateDraft((current) =>
                          current ? applyConfigSnippetToDraft(current, snippet) : current
                        );
                        setSnippetName(snippet.name);
                        setSnippetExport(serializeConfigSnippet(snippet));
                        setSnippetError(null);
                        recordSnippetAudit({
                          type: "config_snippet_import",
                          target: draft.artifactId,
                          detail: `Imported config snippet ${snippet.name}.`
                        });
                      } catch {
                        setSnippetError(copy.configRisk.snippets.importError);
                      }
                    }}
                    type="button"
                  >
                    {copy.configRisk.snippets.actions.applyImportedSnippet}
                  </button>
                </div>
                {savedSnippets.length > 0 ? (
                  <div className="config-snippet-library">
                    <strong>{copy.configRisk.snippets.savedLibrary}</strong>
                    <div className="config-preset-list">
                      {savedSnippets.map((snippet) => (
                        <button
                          className="action-button action-button-secondary config-preset-chip"
                          key={snippet.id}
                          onClick={() => {
                            updateDraft((current) =>
                              current ? applyConfigSnippetToDraft(current, snippet) : current
                            );
                            setSnippetName(snippet.name);
                            setSnippetExport(serializeConfigSnippet(snippet));
                            setSnippetError(null);
                            recordSnippetAudit({
                              type: "config_snippet_apply",
                              target: config.artifactId,
                              detail: `Applied config snippet ${snippet.name}.`
                            });
                          }}
                          type="button"
                        >
                          {snippet.name}
                        </button>
                      ))}
                    </div>
                  </div>
                ) : null}
              </section>
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
