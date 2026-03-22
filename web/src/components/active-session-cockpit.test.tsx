import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../lib/i18n";
import type { SessionDetailRecord } from "../lib/api";

describe("ActiveSessionCockpit", () => {
  it("shows controllable sessions and hides unsupported entries", async () => {
    const { ActiveSessionCockpit } = await import("./active-session-cockpit");

    renderWithI18n(
      <ActiveSessionCockpit
        isRefreshing={false}
        onRefresh={vi.fn()}
        sessions={[
          buildSession({
            sessionId: "ses-attached",
            title: "Resume Codex rollout",
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true,
              lastResponse: "READY from demo continue"
            }
          }),
          buildSession({
            sessionId: "ses-ready",
            title: "Resume Claude relay audit",
            sessionControl: {
              supported: true,
              available: true,
              controller: "claude-code",
              command: "claude",
              attached: false
            }
          }),
          buildSession({
            sessionId: "ses-unsupported",
            title: "Unsupported helper",
            sessionControl: {
              supported: false,
              available: false,
              controller: "unsupported",
              command: "",
              attached: false
            }
          })
        ]}
      />
    );

    expect(
      screen.getByRole("heading", { name: /active session cockpit/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/resume codex rollout/i)).toBeInTheDocument();
    expect(screen.getByText(/ready from demo continue/i)).toBeInTheDocument();
    expect(screen.getByText(/^attached$/i)).toBeInTheDocument();
    expect(screen.getByText(/^ready$/i)).toBeInTheDocument();
    expect(screen.queryByText(/unsupported helper/i)).not.toBeInTheDocument();
  });

  it("disables refresh while a snapshot refresh is in flight", async () => {
    const { ActiveSessionCockpit } = await import("./active-session-cockpit");

    renderWithI18n(
      <ActiveSessionCockpit
        isRefreshing
        onRefresh={vi.fn()}
        sessions={[
          buildSession({
            sessionId: "ses-attached",
            title: "Resume Codex rollout",
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true
            }
          })
        ]}
      />
    );

    expect(
      screen.getByRole("button", { name: /refreshing cockpit/i })
    ).toBeDisabled();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}

function buildSession(
  overrides: Partial<SessionDetailRecord> = {}
): SessionDetailRecord {
  return {
    sessionId: "ses-001",
    title: "Session",
    assistant: "Codex",
    progressState: "In Progress",
    progressPercent: 50,
    lastActivityAt: "2026-03-23T01:00:00.000Z",
    environment: "Windows 11",
    valueScore: 42,
    summary: "Summary",
    projectPath: "C:/Projects/osm",
    sourcePath: "C:/Users/Max/.codex/sessions/demo.jsonl",
    tags: [],
    riskFlags: [],
    keyArtifacts: [],
    transcriptHighlights: [],
    todoItems: [],
    ...overrides
  };
}
