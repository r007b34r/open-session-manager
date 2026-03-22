import type { ConfigRiskRecord, SessionDetailRecord } from "./api";
import { buildModelBreakdown, buildProviderBreakdown } from "./usage-breakdown";

describe("usage breakdown helpers", () => {
  it("aggregates sessions by model with token and cost totals", () => {
    const results = buildModelBreakdown(buildSessions());

    expect(results).toEqual([
      {
        label: "claude-sonnet-4",
        sessionCount: 2,
        totalTokens: 580,
        costUsd: 0.03,
        costSource: "mixed"
      },
      {
        label: "gpt-5",
        sessionCount: 1,
        totalTokens: 200,
        costUsd: 0.01,
        costSource: "reported"
      }
    ]);
  });

  it("aggregates configs by provider and tracks assistant and proxy coverage", () => {
    const results = buildProviderBreakdown(buildConfigs());

    expect(results).toEqual([
      {
        label: "openrouter",
        configCount: 2,
        assistantCount: 2,
        proxyCount: 2
      },
      {
        label: "github",
        configCount: 1,
        assistantCount: 1,
        proxyCount: 1
      }
    ]);
  });
});

function buildSessions(): SessionDetailRecord[] {
  return [
    {
      sessionId: "ses-001",
      title: "Refactor collector",
      assistant: "Codex",
      progressState: "In Progress",
      progressPercent: 65,
      lastActivityAt: "2026-03-15 12:40",
      environment: "WSL: Ubuntu",
      valueScore: 84,
      summary: "Collector manifest work.",
      projectPath: "/home/max/src/open-session-manager",
      sourcePath: "C:/Users/Max/.codex/sessions/2026/03/15/rollout-2026-03-15.jsonl",
      tags: [],
      riskFlags: [],
      keyArtifacts: [],
      transcriptHighlights: [],
      todoItems: [],
      usage: {
        model: "gpt-5",
        inputTokens: 120,
        outputTokens: 80,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        reasoningTokens: 0,
        totalTokens: 200,
        costUsd: 0.01,
        costSource: "reported"
      }
    },
    {
      sessionId: "ses-002",
      title: "Audit relay settings",
      assistant: "Claude Code",
      progressState: "Blocked",
      progressPercent: 15,
      lastActivityAt: "2026-03-14 22:10",
      environment: "Windows 11",
      valueScore: 47,
      summary: "Proxy endpoint still needs remediation.",
      projectPath: "C:/Users/Max/Desktop/ops",
      sourcePath: "C:/Users/Max/.claude/projects/ops/claude-ses-1.jsonl",
      tags: [],
      riskFlags: ["dangerous_permissions"],
      keyArtifacts: [],
      transcriptHighlights: [],
      todoItems: [],
      usage: {
        model: "claude-sonnet-4",
        inputTokens: 250,
        outputTokens: 90,
        cacheReadTokens: 30,
        cacheWriteTokens: 0,
        reasoningTokens: 10,
        totalTokens: 380,
        costUsd: 0.02,
        costSource: "estimated"
      }
    },
    {
      sessionId: "ses-003",
      title: "Cleanup docs",
      assistant: "OpenCode",
      progressState: "Completed",
      progressPercent: 100,
      lastActivityAt: "2026-03-13 08:00",
      environment: "Windows 11",
      valueScore: 70,
      summary: "Finalize the cleanup checklist wording.",
      projectPath: "C:/Users/Max/Desktop/docs",
      sourcePath: "C:/Users/Max/AppData/Local/OpenCode/ses-003.json",
      tags: [],
      riskFlags: [],
      keyArtifacts: [],
      transcriptHighlights: [],
      todoItems: [],
      usage: {
        model: "claude-sonnet-4",
        inputTokens: 120,
        outputTokens: 50,
        cacheReadTokens: 20,
        cacheWriteTokens: 0,
        reasoningTokens: 10,
        totalTokens: 200,
        costUsd: 0.01,
        costSource: "reported"
      }
    }
  ];
}

function buildConfigs(): ConfigRiskRecord[] {
  return [
    {
      artifactId: "cfg-001",
      assistant: "github-copilot-cli",
      scope: "global",
      path: "~/.copilot/config.json",
      provider: "github",
      model: "gpt-5",
      baseUrl: "https://copilot.enterprise-relay.example",
      maskedSecret: "***7890",
      officialOrProxy: "proxy",
      risks: []
    },
    {
      artifactId: "cfg-002",
      assistant: "factory-droid",
      scope: "global",
      path: "~/.factory/settings.local.json",
      provider: "openrouter",
      model: "openrouter/anthropic/claude-sonnet-4",
      baseUrl: "https://factory-relay.example/v1",
      maskedSecret: "***7890",
      officialOrProxy: "proxy",
      risks: []
    },
    {
      artifactId: "cfg-003",
      assistant: "openclaw",
      scope: "project",
      path: "C:/Users/Max/Desktop/docs/.openclaw/openclaw.json",
      provider: "openrouter",
      model: "openrouter/openai/gpt-5-mini",
      baseUrl: "https://openrouter.ai/api/v1",
      maskedSecret: "***4321",
      officialOrProxy: "proxy",
      risks: []
    }
  ];
}
