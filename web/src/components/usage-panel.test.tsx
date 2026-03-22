import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../lib/i18n";
import type {
  ConfigRiskRecord,
  SessionDetailRecord,
  UsageOverviewRecord
} from "../lib/api";
import { UsagePanel } from "./usage-panel";

describe("UsagePanel", () => {
  it("对不可靠的汇总成本显示 Unknown，并保留真实零成本", () => {
    renderWithI18n(
      <UsagePanel
        configs={[]}
        sessions={[]}
        usageOverview={buildUsageOverviewRecord({
          totals: {
            sessionsWithUsage: 2,
            inputTokens: 150,
            outputTokens: 90,
            cacheReadTokens: 0,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 240,
            costUsd: undefined,
            costSource: "unknown"
          },
          assistants: [
            {
              assistant: "Codex",
              sessionCount: 1,
              inputTokens: 120,
              outputTokens: 80,
              cacheReadTokens: 0,
              cacheWriteTokens: 0,
              reasoningTokens: 0,
              totalTokens: 200,
              costSource: "unknown"
            },
            {
              assistant: "OpenCode",
              sessionCount: 1,
              inputTokens: 30,
              outputTokens: 10,
              cacheReadTokens: 0,
              cacheWriteTokens: 0,
              reasoningTokens: 0,
              totalTokens: 40,
              costUsd: 0,
              costSource: "reported"
            }
          ]
        })}
        usageTimeline={[]}
      />
    );

    expect(screen.getAllByText("Unknown")).toHaveLength(2);
    expect(screen.getByText(/\$0\.00/i)).toBeInTheDocument();
    expect(screen.queryByText(/\$nan/i)).not.toBeInTheDocument();
  });

  it("展示成本来源和按天趋势，不把未知成本伪装成零", () => {
    renderWithI18n(
      <UsagePanel
        configs={buildConfigs()}
        sessions={buildSessions()}
        usageOverview={buildUsageOverviewRecord({
          totals: {
            sessionsWithUsage: 2,
            inputTokens: 1579,
            outputTokens: 690,
            cacheReadTokens: 967,
            cacheWriteTokens: 144,
            reasoningTokens: 45,
            totalTokens: 3425,
            costUsd: undefined,
            costSource: "unknown"
          },
          assistants: [
            {
              assistant: "Claude Code",
              sessionCount: 1,
              inputTokens: 1234,
              outputTokens: 567,
              cacheReadTokens: 890,
              cacheWriteTokens: 144,
              reasoningTokens: 0,
              totalTokens: 2835,
              costUsd: 0.01301,
              costSource: "estimated"
            },
            {
              assistant: "OpenClaw",
              sessionCount: 1,
              inputTokens: 345,
              outputTokens: 123,
              cacheReadTokens: 77,
              cacheWriteTokens: 0,
              reasoningTokens: 45,
              totalTokens: 590,
              costUsd: 0.02,
              costSource: "reported"
            }
          ]
        })}
        usageTimeline={[
          {
            date: "2026-03-15",
            sessionsWithUsage: 2,
            inputTokens: 1579,
            outputTokens: 690,
            cacheReadTokens: 967,
            cacheWriteTokens: 144,
            reasoningTokens: 45,
            totalTokens: 3425,
            costUsd: undefined,
            costSource: "unknown"
          },
          {
            date: "2026-03-16",
            sessionsWithUsage: 1,
            inputTokens: 1234,
            outputTokens: 567,
            cacheReadTokens: 890,
            cacheWriteTokens: 144,
            reasoningTokens: 0,
            totalTokens: 2835,
            costUsd: 0.01301,
            costSource: "estimated"
          }
        ]}
      />
    );

    expect(screen.getAllByText(/estimated from local price catalog/i)).toHaveLength(3);
    expect(screen.getAllByText(/reported by session log/i)).toHaveLength(2);
    expect(screen.getByText(/daily timeline/i)).toBeInTheDocument();
    expect(screen.getByText("2026-03-15")).toBeInTheDocument();
    expect(screen.getByText("2026-03-16")).toBeInTheDocument();
    expect(screen.getByText(/cost unavailable/i)).toBeInTheDocument();
    expect(screen.getByText(/model breakdown/i)).toBeInTheDocument();
    expect(screen.getByText(/platform breakdown/i)).toBeInTheDocument();
    expect(screen.getByText("claude-sonnet-4")).toBeInTheDocument();
    expect(screen.getByText("openrouter")).toBeInTheDocument();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}

function buildUsageOverviewRecord(
  overrides: Partial<UsageOverviewRecord> = {}
): UsageOverviewRecord {
  return {
    totals: {
      sessionsWithUsage: 0,
      inputTokens: 0,
      outputTokens: 0,
      cacheReadTokens: 0,
      cacheWriteTokens: 0,
      reasoningTokens: 0,
      totalTokens: 0,
      costUsd: 0,
      costSource: "reported",
      ...(overrides.totals ?? {})
    },
    assistants: overrides.assistants ?? []
  } as UsageOverviewRecord;
}

function buildSessions(): SessionDetailRecord[] {
  return [
    {
      sessionId: "ses-001",
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
      riskFlags: [],
      keyArtifacts: [],
      transcriptHighlights: [],
      todoItems: [],
      usage: {
        model: "claude-sonnet-4",
        inputTokens: 1234,
        outputTokens: 567,
        cacheReadTokens: 890,
        cacheWriteTokens: 144,
        reasoningTokens: 0,
        totalTokens: 2835,
        costUsd: 0.01301,
        costSource: "estimated"
      }
    },
    {
      sessionId: "ses-002",
      title: "Summarize queue",
      assistant: "Codex",
      progressState: "Completed",
      progressPercent: 100,
      lastActivityAt: "2026-03-15 10:00",
      environment: "WSL: Ubuntu",
      valueScore: 72,
      summary: "Queue handoff complete.",
      projectPath: "/home/max/src/open-session-manager",
      sourcePath: "C:/Users/Max/.codex/sessions/2026/03/15/rollout.jsonl",
      tags: [],
      riskFlags: [],
      keyArtifacts: [],
      transcriptHighlights: [],
      todoItems: [],
      usage: {
        model: "gpt-5",
        inputTokens: 345,
        outputTokens: 123,
        cacheReadTokens: 77,
        cacheWriteTokens: 0,
        reasoningTokens: 45,
        totalTokens: 590,
        costUsd: 0.02,
        costSource: "reported"
      }
    }
  ];
}

function buildConfigs(): ConfigRiskRecord[] {
  return [
    {
      artifactId: "cfg-001",
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
      artifactId: "cfg-002",
      assistant: "github-copilot-cli",
      scope: "global",
      path: "~/.copilot/config.json",
      provider: "github",
      model: "gpt-5",
      baseUrl: "https://copilot.enterprise-relay.example",
      maskedSecret: "***7890",
      officialOrProxy: "proxy",
      risks: []
    }
  ];
}
