import type { ConfigRiskRecord, ConfigWritebackInput } from "./api";
import { buildConfigReview } from "./review-flow";

describe("review flow helpers", () => {
  it("builds masked config diffs and warnings for sensitive changes", () => {
    const review = buildConfigReview(buildConfig(), buildDraft());

    expect(review.entries).toEqual([
      {
        field: "model",
        before: "gpt-5",
        after: "gpt-5-mini",
        severity: "safe"
      },
      {
        field: "baseUrl",
        before: "https://copilot.enterprise-relay.example",
        after: "https://github.com/api/copilot",
        severity: "warning"
      },
      {
        field: "secret",
        before: "***7890",
        after: "***4321",
        severity: "warning"
      }
    ]);
    expect(review.warnings).toEqual([
      "endpoint_changed",
      "secret_changed",
      "existing_risks"
    ]);
  });
});

function buildConfig(): ConfigRiskRecord {
  return {
    artifactId: "cfg-003",
    assistant: "GitHub Copilot CLI",
    scope: "Global",
    path: "~/.copilot/config.json",
    provider: "github",
    model: "gpt-5",
    baseUrl: "https://copilot.enterprise-relay.example",
    maskedSecret: "***7890",
    officialOrProxy: "Proxy",
    risks: ["third_party_base_url", "dangerous_permissions"]
  };
}

function buildDraft(): ConfigWritebackInput {
  return {
    artifactId: "cfg-003",
    assistant: "GitHub Copilot CLI",
    scope: "Global",
    path: "~/.copilot/config.json",
    provider: "github",
    model: "gpt-5-mini",
    baseUrl: "https://github.com/api/copilot",
    secret: "ghu_new_secret_123454321"
  };
}
