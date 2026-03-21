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
            costUsd: undefined
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
              totalTokens: 200
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
              costUsd: 0
            }
          ]
        })}
      />
    );

    expect(screen.getAllByText("Unknown")).toHaveLength(2);
    expect(screen.getByText(/\$0\.00/i)).toBeInTheDocument();
    expect(screen.queryByText(/\$nan/i)).not.toBeInTheDocument();
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
      ...(overrides.totals ?? {})
    },
    assistants: overrides.assistants ?? []
  } as UsageOverviewRecord;
}
