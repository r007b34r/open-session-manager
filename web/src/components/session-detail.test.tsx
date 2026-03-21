import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../lib/i18n";
import type { SessionDetailRecord } from "../lib/api";
import { SessionDetail } from "./session-detail";

describe("SessionDetail", () => {
  it("未知成本时明确显示 Unknown，而不伪装成 $0.00", () => {
    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          usage: {
            model: "gpt-5-codex",
            inputTokens: 120,
            outputTokens: 80,
            cacheReadTokens: 0,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 200
          }
        })}
      />
    );

    expect(screen.getByText(/cost \(usd\): unknown/i)).toBeInTheDocument();
    expect(screen.queryByText(/\$0\.00/i)).not.toBeInTheDocument();
  });

  it("真实零成本时仍然显示 $0.00", () => {
    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          usage: {
            model: "gpt-5-codex",
            inputTokens: 120,
            outputTokens: 80,
            cacheReadTokens: 0,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 200,
            costUsd: 0
          }
        })}
      />
    );

    expect(screen.getByText(/cost \(usd\): \$0\.00/i)).toBeInTheDocument();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}

function buildSessionDetailRecord(
  overrides: Partial<SessionDetailRecord> = {}
): SessionDetailRecord {
  return {
    sessionId: "ses-001",
    title: "Unknown cost detail",
    assistant: "Codex",
    progressState: "In Progress",
    progressPercent: 65,
    lastActivityAt: "2026-03-15 12:40",
    environment: "WSL: Ubuntu",
    valueScore: 84,
    summary: "Collector still needs a stable manifest layer.",
    projectPath: "/home/max/src/open-session-manager",
    sourcePath: "C:/Users/Max/.codex/sessions/demo.jsonl",
    tags: ["wsl", "collector"],
    riskFlags: [],
    keyArtifacts: ["Defined distro handshake checkpoints"],
    transcriptHighlights: [],
    todoItems: [],
    ...overrides
  } as SessionDetailRecord;
}
