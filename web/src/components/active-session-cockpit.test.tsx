import { render, screen, within } from "@testing-library/react";
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

  it("shows busy, waiting, and idle runtime states", async () => {
    const { ActiveSessionCockpit } = await import("./active-session-cockpit");

    renderWithI18n(
      <ActiveSessionCockpit
        isRefreshing={false}
        onRefresh={vi.fn()}
        sessions={[
          buildSession({
            sessionId: "ses-busy",
            title: "Busy Codex rollout",
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true,
              runtimeState: "busy"
            } as any
          }),
          buildSession({
            sessionId: "ses-waiting",
            title: "Waiting Claude audit",
            sessionControl: {
              supported: true,
              available: true,
              controller: "claude-code",
              command: "claude",
              attached: true,
              runtimeState: "waiting"
            } as any
          }),
          buildSession({
            sessionId: "ses-idle",
            title: "Idle cleanup review",
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true,
              runtimeState: "idle"
            } as any
          })
        ]}
      />
    );

    expect(screen.getByText(/^busy$/i)).toBeInTheDocument();
    expect(screen.getByText(/^waiting$/i)).toBeInTheDocument();
    expect(screen.getByText(/^idle$/i)).toBeInTheDocument();
  });

  it("shows paused live HUD details", async () => {
    const { ActiveSessionCockpit } = await import("./active-session-cockpit");

    renderWithI18n(
      <ActiveSessionCockpit
        isRefreshing={false}
        onRefresh={vi.fn()}
        sessions={[
          buildSession({
            sessionId: "ses-paused",
            title: "Paused Codex rollout",
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true,
              runtimeState: "paused",
              processState: "paused",
              processId: 4321,
              runtimeSeconds: 1200,
              eventCount: 7,
              totalTokens: 154
            } as any
          })
        ]}
      />
    );

    const pausedCard = screen.getByText(/paused codex rollout/i).closest("article");

    expect(pausedCard).not.toBeNull();
    expect(within(pausedCard as HTMLElement).getByText(/^paused$/i)).toBeInTheDocument();

    const processIdRow = within(pausedCard as HTMLElement)
      .getByText(/^process id$/i)
      .closest("div");
    const runtimeRow = within(pausedCard as HTMLElement)
      .getByText(/^runtime \(sec\)$/i)
      .closest("div");
    const eventsLine = within(pausedCard as HTMLElement)
      .getByText(/^events$/i)
      .closest("p");
    const tokensLine = within(pausedCard as HTMLElement)
      .getByText(/^tokens$/i)
      .closest("p");

    expect(processIdRow).not.toBeNull();
    expect(runtimeRow).not.toBeNull();
    expect(eventsLine).not.toBeNull();
    expect(tokensLine).not.toBeNull();
    expect(processIdRow).toHaveTextContent(/^Process ID\s*4321$/i);
    expect(runtimeRow).toHaveTextContent(/^Runtime \(sec\)\s*1200$/i);
    expect(eventsLine).toHaveTextContent(/^Events:\s*7$/i);
    expect(tokensLine).toHaveTextContent(/^Tokens:\s*154$/i);
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
