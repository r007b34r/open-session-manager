import type { ConfigRiskRecord, ConfigWritebackInput } from "./api";

export type ReviewDiffField = "provider" | "model" | "baseUrl" | "secret";
export type ReviewDiffSeverity = "safe" | "warning";
export type ReviewWarningCode =
  | "provider_changed"
  | "endpoint_changed"
  | "secret_changed"
  | "existing_risks";

export type ReviewDiffEntry = {
  field: ReviewDiffField;
  before: string;
  after: string;
  severity: ReviewDiffSeverity;
};

export type ConfigReview = {
  entries: ReviewDiffEntry[];
  warnings: ReviewWarningCode[];
};

export function buildConfigReview(
  config: ConfigRiskRecord,
  draft: ConfigWritebackInput
): ConfigReview {
  const entries: ReviewDiffEntry[] = [];
  const warnings: ReviewWarningCode[] = [];

  pushEntry(entries, "provider", config.provider, draft.provider, "warning");
  pushEntry(entries, "model", config.model, draft.model, "safe");
  pushEntry(entries, "baseUrl", config.baseUrl, draft.baseUrl, "warning");

  if (config.provider.trim() !== draft.provider.trim()) {
    warnings.push("provider_changed");
  }

  if (config.baseUrl.trim() !== draft.baseUrl.trim()) {
    warnings.push("endpoint_changed");
  }

  const nextSecret = normalizeOptionalValue(draft.secret);
  if (nextSecret) {
    entries.push({
      field: "secret",
      before: normalizeDisplayValue(config.maskedSecret),
      after: maskSecret(nextSecret),
      severity: "warning"
    });
    warnings.push("secret_changed");
  }

  if (config.risks.length > 0) {
    warnings.push("existing_risks");
  }

  return {
    entries,
    warnings
  };
}

function pushEntry(
  entries: ReviewDiffEntry[],
  field: ReviewDiffField,
  before: string | undefined,
  after: string | undefined,
  severity: ReviewDiffSeverity
) {
  const normalizedBefore = normalizeDisplayValue(before);
  const normalizedAfter = normalizeDisplayValue(after);

  if (normalizedBefore === normalizedAfter) {
    return;
  }

  entries.push({
    field,
    before: normalizedBefore,
    after: normalizedAfter,
    severity
  });
}

function normalizeDisplayValue(value: string | undefined) {
  const normalized = value?.trim();
  return normalized ? normalized : "Not set";
}

function normalizeOptionalValue(value: string | undefined) {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
}

function maskSecret(value: string) {
  const visible = value.slice(-4);
  return visible ? `***${visible}` : "***";
}
