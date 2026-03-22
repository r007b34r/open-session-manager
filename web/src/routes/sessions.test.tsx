import { act, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";

import type { DashboardRuntime, SessionDetailRecord } from "../lib/api";
import { I18nProvider } from "../lib/i18n";
import { SessionsRoute } from "./sessions";

describe("SessionsRoute", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("keeps previous results visible while a search update is pending", async () => {
    vi.useFakeTimers();

    renderWithI18n(
      <SessionsRoute
        exportedSessionIds={new Set()}
        latestMarkdownExportPaths={new Map()}
        runtime={buildRuntime()}
        sessions={buildSessions()}
      />
    );

    const searchbox = screen.getByRole("searchbox", { name: /search sessions/i });
    fireEvent.change(searchbox, { target: { value: "ops" } });

    expect(
      screen.getByRole("button", { name: /refactor wsl collector handshake/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/updating matches/i)).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(220);
    });

    expect(
      screen.queryByRole("button", { name: /refactor wsl collector handshake/i })
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    ).toBeInTheDocument();
  });

  it("cancels an older pending search when the user types a newer query", async () => {
    vi.useFakeTimers();

    renderWithI18n(
      <SessionsRoute
        exportedSessionIds={new Set()}
        latestMarkdownExportPaths={new Map()}
        runtime={buildRuntime()}
        sessions={buildSessions()}
      />
    );

    const searchbox = screen.getByRole("searchbox", { name: /search sessions/i });
    fireEvent.change(searchbox, { target: { value: "ref" } });

    await act(async () => {
      vi.advanceTimersByTime(150);
    });

    fireEvent.change(searchbox, { target: { value: "ops" } });

    await act(async () => {
      vi.advanceTimersByTime(60);
    });

    expect(
      screen.getByRole("button", { name: /refactor wsl collector handshake/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/updating matches/i)).toBeInTheDocument();

    await act(async () => {
      vi.advanceTimersByTime(160);
    });

    expect(
      screen.queryByRole("button", { name: /refactor wsl collector handshake/i })
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    ).toBeInTheDocument();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}

function buildRuntime(): DashboardRuntime {
  return {
    auditDbPath: "C:/Users/Max/AppData/Local/OpenSessionManager/audit/audit.db",
    exportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
    defaultExportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
    exportRootSource: "default",
    quarantineRoot: "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine",
    preferencesPath: "C:/Users/Max/AppData/Local/OpenSessionManager/preferences.json"
  };
}

function buildSessions(): SessionDetailRecord[] {
  return [
    {
      sessionId: "ses-001",
      title: "Refactor WSL collector handshake",
      assistant: "Codex",
      progressState: "In Progress",
      progressPercent: 65,
      lastActivityAt: "2026-03-15 12:40",
      environment: "WSL: Ubuntu",
      valueScore: 84,
      summary: "Collector manifest work.",
      projectPath: "/home/max/src/open-session-manager",
      sourcePath: "C:/Users/Max/.codex/sessions/2026/03/15/rollout-2026-03-15.jsonl",
      tags: ["wsl", "collector"],
      riskFlags: [],
      keyArtifacts: ["Finalize manifest framing"],
      transcriptHighlights: [],
      todoItems: []
    },
    {
      sessionId: "ses-002",
      title: "Audit Anthropic relay settings",
      assistant: "Claude Code",
      progressState: "Blocked",
      progressPercent: 15,
      lastActivityAt: "2026-03-14 22:10",
      environment: "Windows 11",
      valueScore: 47,
      summary: "Proxy endpoint still needs remediation.",
      projectPath: "C:/Users/Max/Desktop/ops",
      sourcePath: "C:/Users/Max/.claude/projects/ops/claude-ses-1.jsonl",
      tags: ["relay", "risk"],
      riskFlags: ["dangerous_permissions"],
      keyArtifacts: ["Captured hook chain"],
      transcriptHighlights: [],
      todoItems: []
    }
  ];
}
