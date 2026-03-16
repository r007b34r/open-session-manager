import { render, screen } from "@testing-library/react";

describe("ConfigRiskPanel", () => {
  it("renders masked values and risk badges for risky config entries", async () => {
    const { ConfigRiskPanel } = await import("./config-risk-panel");

    render(
      <ConfigRiskPanel
        configs={[
          {
            artifactId: "cfg-001",
            assistant: "Codex",
            scope: "Global",
            path: "~/.codex/config.toml",
            provider: "cch",
            model: "gpt-5-codex",
            baseUrl: "https://relay.cch.example/v1",
            maskedSecret: "***6789",
            officialOrProxy: "Proxy",
            risks: ["third_party_base_url", "dangerous_sandbox"]
          },
          {
            artifactId: "cfg-002",
            assistant: "OpenCode",
            scope: "Project",
            path: ".opencode/opencode.json",
            provider: "openrouter",
            baseUrl: "https://openrouter.ai/api/v1",
            maskedSecret: "***3456",
            officialOrProxy: "Proxy",
            risks: ["dangerous_permissions"]
          },
          {
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
          },
          {
            artifactId: "cfg-004",
            assistant: "Factory Droid",
            scope: "Global",
            path: "~/.factory/settings.local.json",
            provider: "openrouter",
            model: "openrouter/anthropic/claude-sonnet-4",
            baseUrl: "https://factory-relay.example/v1",
            maskedSecret: "***7890",
            officialOrProxy: "Proxy",
            risks: ["third_party_provider", "dangerous_permissions"]
          }
        ]}
      />
    );

    expect(
      screen.getByRole("heading", { name: /config risk center/i })
    ).toBeInTheDocument();
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getAllByText("Proxy")).toHaveLength(4);
    expect(screen.getByText("GitHub Copilot CLI")).toBeInTheDocument();
    expect(screen.getByText("Factory Droid")).toBeInTheDocument();
    expect(screen.getByText("***6789")).toBeInTheDocument();
    expect(screen.getAllByText("***7890")).toHaveLength(2);
    expect(screen.getAllByText(/model/i)).toHaveLength(3);
    expect(screen.getByText("gpt-5-codex")).toBeInTheDocument();
    expect(screen.getByText("gpt-5")).toBeInTheDocument();
    expect(screen.getAllByText(/third_party_base_url/i)).toHaveLength(2);
    expect(screen.getByText(/dangerous_sandbox/i)).toBeInTheDocument();
  });
});
