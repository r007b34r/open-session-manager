import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

describe("ConfigRiskPanel", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

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

  it("requires reviewing a risky config change before submitting the writeback payload", async () => {
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

    await user.click(screen.getByRole("button", { name: /review changes/i }));

    expect(onSaveConfig).not.toHaveBeenCalled();
    expect(screen.getByRole("heading", { name: /review changes/i })).toBeInTheDocument();
    expect(screen.getByText("gpt-5-mini")).toBeInTheDocument();
    expect(screen.getByText("https://github.com/api/copilot")).toBeInTheDocument();

    const applyButton = screen.getByRole("button", { name: /apply reviewed changes/i });
    expect(applyButton).toBeDisabled();

    await user.click(
      screen.getByRole("checkbox", {
        name: /i reviewed the masked diff and want to apply it/i
      })
    );
    await user.click(applyButton);

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

  it("allows editing qwen cli configs from the risk panel", async () => {
    const user = userEvent.setup();
    const onSaveConfig = vi.fn();
    const { ConfigRiskPanel } = await import("./config-risk-panel");

    render(
      <ConfigRiskPanel
        canEditConfigs
        configs={[
          {
            artifactId: "cfg-005",
            assistant: "Qwen CLI",
            scope: "Global",
            path: "~/.qwen/settings.json",
            provider: "openai",
            model: "qwen3-coder-plus",
            baseUrl: "https://qwen-relay.example/v1",
            maskedSecret: "***7890",
            officialOrProxy: "Proxy",
            risks: ["third_party_base_url"]
          }
        ]}
        onSaveConfig={onSaveConfig}
      />
    );

    await user.click(screen.getByRole("button", { name: /edit config/i }));
    await user.clear(screen.getByLabelText(/^model$/i));
    await user.type(screen.getByLabelText(/^model$/i), "qwen3-coder-max");
    await user.clear(screen.getByLabelText(/endpoint/i));
    await user.type(screen.getByLabelText(/endpoint/i), "https://api.openai.com/v1");
    await user.type(screen.getByLabelText(/new key/i), "sk-qwen-new-123454321");
    await user.click(screen.getByRole("button", { name: /review changes/i }));
    await user.click(
      screen.getByRole("checkbox", {
        name: /i reviewed the masked diff and want to apply it/i
      })
    );
    await user.click(screen.getByRole("button", { name: /apply reviewed changes/i }));

    expect(onSaveConfig).toHaveBeenCalledWith({
      artifactId: "cfg-005",
      assistant: "Qwen CLI",
      scope: "Global",
      path: "~/.qwen/settings.json",
      provider: "openai",
      model: "qwen3-coder-max",
      baseUrl: "https://api.openai.com/v1",
      secret: "sk-qwen-new-123454321"
    });
  });

  it("saves a reusable snippet and reapplies it back to the current draft", async () => {
    const user = userEvent.setup();
    const onAuditEvent = vi.fn();
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
            baseUrl: "https://api.githubcopilot.com",
            maskedSecret: "***7890",
            officialOrProxy: "Official",
            risks: []
          }
        ]}
        onAuditEvent={onAuditEvent}
      />
    );

    await user.click(screen.getByRole("button", { name: /edit config/i }));
    await user.type(screen.getByLabelText(/snippet name/i), "Shared GitHub");
    await user.click(screen.getByRole("button", { name: /save snippet/i }));

    expect(onAuditEvent).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "config_snippet_save",
        target: "cfg-003"
      })
    );

    await user.clear(screen.getByLabelText(/^model$/i));
    await user.type(screen.getByLabelText(/^model$/i), "gpt-5-mini");
    await user.clear(screen.getByLabelText(/endpoint/i));
    await user.type(screen.getByLabelText(/endpoint/i), "https://example.invalid/v1");
    await user.click(screen.getByRole("button", { name: /shared github/i }));

    expect(screen.getByLabelText(/^model$/i)).toHaveValue("gpt-5");
    expect(screen.getByLabelText(/endpoint/i)).toHaveValue(
      "https://api.githubcopilot.com"
    );
  });

  it("exports snippet JSON and imports a snippet payload back into the draft", async () => {
    const user = userEvent.setup();
    const onAuditEvent = vi.fn();
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
            baseUrl: "https://openrouter.ai/api/v1",
            maskedSecret: "***7890",
            officialOrProxy: "Proxy",
            risks: ["third_party_provider"]
          }
        ]}
        onAuditEvent={onAuditEvent}
      />
    );

    await user.click(screen.getByRole("button", { name: /edit config/i }));
    await user.type(screen.getByLabelText(/snippet name/i), "Shared Router");
    await user.click(screen.getByRole("button", { name: /prepare export/i }));

    expect(
      (screen.getByLabelText(/snippet export json/i) as HTMLTextAreaElement).value
    ).toContain('"name": "Shared Router"');
    expect(onAuditEvent).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "config_snippet_export",
        target: "cfg-004"
      })
    );

    const snippetImportInput = screen.getByLabelText(/snippet import json/i);
    await user.clear(snippetImportInput);
    await user.click(snippetImportInput);
    await user.paste(
      '{"version":1,"name":"OpenAI Shared","provider":"openai","model":"gpt-5-mini","baseUrl":"https://api.openai.com/v1","originAssistant":"Factory Droid","createdAt":"2026-03-23T02:00:00.000Z"}'
    );
    await user.click(screen.getByRole("button", { name: /apply imported snippet/i }));

    expect(screen.getByLabelText(/provider/i)).toHaveValue("openai");
    expect(screen.getByLabelText(/^model$/i)).toHaveValue("gpt-5-mini");
    expect(screen.getByLabelText(/endpoint/i)).toHaveValue("https://api.openai.com/v1");
    expect(onAuditEvent).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "config_snippet_import",
        target: "cfg-004"
      })
    );
  });

  it("renders a viewer for discovered MCP server configs", async () => {
    const { ConfigRiskPanel } = await import("./config-risk-panel");

    render(
      <ConfigRiskPanel
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
            risks: ["third_party_base_url", "dangerous_permissions"],
            mcpServers: [
              {
                serverId: "cfg-003:filesystem",
                name: "filesystem",
                enabled: true,
                status: "configured",
                transport: "stdio",
                command: "node",
                args: ["mcp-filesystem.js"],
                configJson:
                  '{\n  "command": "node",\n  "args": ["mcp-filesystem.js"]\n}'
              }
            ]
          },
          {
            artifactId: "cfg-004",
            assistant: "OpenCode",
            scope: "Project",
            path: ".opencode/opencode.json",
            provider: "openrouter",
            model: "openrouter/anthropic/claude-sonnet-4",
            baseUrl: "https://openrouter.ai/api/v1",
            maskedSecret: "***3456",
            officialOrProxy: "Proxy",
            risks: ["dangerous_permissions"],
            mcpServers: [
              {
                serverId: "cfg-004:filesystem",
                name: "filesystem",
                enabled: true,
                status: "enabled",
                transport: "embedded",
                args: [],
                configJson: '{\n  "enabled": true\n}'
              }
            ]
          }
        ]}
      />
    );

    expect(
      screen.getByRole("heading", { name: /mcp server viewer/i })
    ).toBeInTheDocument();
    expect(screen.getAllByText("filesystem")).toHaveLength(2);
    expect(screen.getAllByText("GitHub Copilot CLI")).toHaveLength(2);
    expect(screen.getByText("node mcp-filesystem.js")).toBeInTheDocument();
    expect(screen.getByText("Configured")).toBeInTheDocument();
    expect(screen.getByText("Embedded")).toBeInTheDocument();
    expect(
      screen.getByText((content) => content.includes('"enabled": true'))
    ).toBeInTheDocument();
  });

  it("groups project-scoped configs under the same project root", async () => {
    const { ConfigRiskPanel } = await import("./config-risk-panel");

    render(
      <ConfigRiskPanel
        configs={[
          {
            artifactId: "cfg-201",
            assistant: "GitHub Copilot CLI",
            scope: "Project",
            path: "C:/Projects/alpha/.github/copilot/settings.local.json",
            provider: "github",
            model: "gpt-5",
            baseUrl: "https://api.githubcopilot.com",
            maskedSecret: "***7890",
            officialOrProxy: "Official",
            risks: [],
            mcpServers: []
          },
          {
            artifactId: "cfg-202",
            assistant: "Factory Droid",
            scope: "Project",
            path: "C:/Projects/alpha/.factory/settings.local.json",
            provider: "openrouter",
            model: "openrouter/anthropic/claude-sonnet-4",
            baseUrl: "https://openrouter.ai/api/v1",
            maskedSecret: "***3456",
            officialOrProxy: "Proxy",
            risks: ["third_party_provider"],
            mcpServers: []
          }
        ]}
      />
    );

    const alphaGroup = screen
      .getByRole("heading", { name: "C:/Projects/alpha" })
      .closest("section");

    expect(alphaGroup).not.toBeNull();
    expect(
      within(alphaGroup as HTMLElement).getByText(
        "C:/Projects/alpha/.github/copilot/settings.local.json"
      )
    ).toBeInTheDocument();
    expect(
      within(alphaGroup as HTMLElement).getByText(
        "C:/Projects/alpha/.factory/settings.local.json"
      )
    ).toBeInTheDocument();
  });
});
