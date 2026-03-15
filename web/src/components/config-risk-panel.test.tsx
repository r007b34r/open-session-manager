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
          }
        ]}
      />
    );

    expect(
      screen.getByRole("heading", { name: /config risk center/i })
    ).toBeInTheDocument();
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getAllByText("Proxy")).toHaveLength(2);
    expect(screen.getByText("***6789")).toBeInTheDocument();
    expect(screen.getByText(/third_party_base_url/i)).toBeInTheDocument();
    expect(screen.getByText(/dangerous_sandbox/i)).toBeInTheDocument();
  });
});
