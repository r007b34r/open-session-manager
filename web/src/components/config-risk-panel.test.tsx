import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

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

  it("allows editing a supported config card and submits the writeback payload", async () => {
    const user = userEvent.setup();
    const onSaveConfig = vi.fn();
    const { ConfigRiskPanel } = await import("./config-risk-panel");

    render(
      <ConfigRiskPanel
        canEditConfigs
        configs={[
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
          }
        ]}
        onSaveConfig={onSaveConfig}
      />
    );

    await user.click(screen.getByRole("button", { name: /edit config/i }));
    await user.clear(screen.getByLabelText(/^model$/i));
    await user.type(screen.getByLabelText(/^model$/i), "gpt-5-mini");
    await user.clear(screen.getByLabelText(/endpoint/i));
    await user.type(
      screen.getByLabelText(/endpoint/i),
      "https://github.com/api/copilot"
    );
    await user.type(
      screen.getByLabelText(/new key/i),
      "ghu_new_secret_123454321"
    );
    await user.click(screen.getByRole("button", { name: /save config/i }));

    expect(onSaveConfig).toHaveBeenCalledWith({
      artifactId: "cfg-003",
      assistant: "GitHub Copilot CLI",
      scope: "Global",
      path: "~/.copilot/config.json",
      provider: "github",
      model: "gpt-5-mini",
      baseUrl: "https://github.com/api/copilot",
      secret: "ghu_new_secret_123454321"
    });
  });

  it("shows provider presets for supported assistants and can revert a draft back to detected values", async () => {
    const user = userEvent.setup();
    const { ConfigRiskPanel } = await import("./config-risk-panel");

    render(
      <ConfigRiskPanel
        canEditConfigs
        configs={[
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

    await user.click(screen.getByRole("button", { name: /edit config/i }));

    expect(screen.getByText(/provider presets/i)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /openai official/i }));

    expect(screen.getByLabelText(/provider/i)).toHaveValue("openai");
    expect(screen.getByLabelText(/^model$/i)).toHaveValue("gpt-5-mini");
    expect(screen.getByLabelText(/endpoint/i)).toHaveValue("https://api.openai.com/v1");

    await user.click(screen.getByRole("button", { name: /restore detected values/i }));

    expect(screen.getByLabelText(/provider/i)).toHaveValue("openrouter");
    expect(screen.getByLabelText(/^model$/i)).toHaveValue(
      "openrouter/anthropic/claude-sonnet-4"
    );
    expect(screen.getByLabelText(/endpoint/i)).toHaveValue(
      "https://factory-relay.example/v1"
    );
  });
});
