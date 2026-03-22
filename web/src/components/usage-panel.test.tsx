import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../lib/i18n";
import type { UsageOverviewRecord } from "../lib/api";
import { UsagePanel } from "./usage-panel";

describe("UsagePanel", () => {
  it("对不可靠的汇总成本显示 Unknown，并保留真实零成本", () => {
    renderWithI18n(
      <UsagePanel
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

    expect(screen.getAllByText(/estimated from local price catalog/i)).toHaveLength(2);
    expect(screen.getByText(/reported by session log/i)).toBeInTheDocument();
    expect(screen.getByText(/daily timeline/i)).toBeInTheDocument();
    expect(screen.getByText("2026-03-15")).toBeInTheDocument();
    expect(screen.getByText("2026-03-16")).toBeInTheDocument();
    expect(screen.getByText(/cost unavailable/i)).toBeInTheDocument();
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
