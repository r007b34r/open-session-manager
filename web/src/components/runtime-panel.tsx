import { useEffect, useState } from "react";

import type { DashboardRuntime } from "../lib/api";
import { useI18n } from "../lib/i18n";

type RuntimePanelProps = {
  runtime: DashboardRuntime;
  onSaveExportRoot?: (path: string) => void;
  onResetExportRoot?: () => void;
};

export function RuntimePanel({
  runtime,
  onSaveExportRoot,
  onResetExportRoot
}: RuntimePanelProps) {
  const { copy } = useI18n();
  const [draftExportRoot, setDraftExportRoot] = useState(runtime.exportRoot);

  useEffect(() => {
    setDraftExportRoot(runtime.exportRoot);
  }, [runtime.exportRoot]);

  const normalizedDraft = draftExportRoot.trim();
  const hasChanges = normalizedDraft.length > 0 && normalizedDraft !== runtime.exportRoot;
  const hasCustomExportRoot = runtime.exportRootSource === "custom";

  return (
    <section className="panel runtime-panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.runtimePanel.kicker}</p>
          <h2>{copy.runtimePanel.title}</h2>
        </div>
        <p className="panel-copy">{copy.runtimePanel.description}</p>
      </div>

      <div className="runtime-grid">
        <section className="runtime-card">
          <label className="search-label" htmlFor="export-root-input">
            {copy.runtimePanel.exportRootLabel}
          </label>
          <input
            className="search-input"
            id="export-root-input"
            onChange={(event) => setDraftExportRoot(event.target.value)}
            type="text"
            value={draftExportRoot}
          />
          <p className="runtime-note">
            {hasCustomExportRoot
              ? copy.runtimePanel.customExportRootHint
              : copy.runtimePanel.defaultExportRootHint}
          </p>
          <div className="action-row">
            <button
              className="action-button action-button-primary"
              disabled={!hasChanges}
              onClick={() => onSaveExportRoot?.(normalizedDraft)}
              type="button"
            >
              {copy.runtimePanel.actions.saveExportRoot}
            </button>
            <button
              className="action-button action-button-secondary"
              disabled={!hasCustomExportRoot}
              onClick={() => onResetExportRoot?.()}
              type="button"
            >
              {copy.runtimePanel.actions.resetExportRoot}
            </button>
          </div>
        </section>

        <section className="runtime-card">
          <dl className="runtime-meta">
            <div>
              <dt>{copy.runtimePanel.fields.exportRoot}</dt>
              <dd>{runtime.exportRoot}</dd>
            </div>
            <div>
              <dt>{copy.runtimePanel.fields.auditDb}</dt>
              <dd>{runtime.auditDbPath}</dd>
            </div>
            <div>
              <dt>{copy.runtimePanel.fields.quarantineRoot}</dt>
              <dd>{runtime.quarantineRoot}</dd>
            </div>
            <div>
              <dt>{copy.runtimePanel.fields.preferencesFile}</dt>
              <dd>{runtime.preferencesPath}</dd>
            </div>
          </dl>
        </section>
      </div>
    </section>
  );
}
