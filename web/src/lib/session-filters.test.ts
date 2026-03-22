import type { SessionDetailRecord } from "./api";
import { applySessionFilters, type SessionFilterState } from "./session-filters";

describe("applySessionFilters", () => {
  it("supports combining assistant, project, risk, export, and control filters", () => {
    const sessions = buildSessions();
    const filters: SessionFilterState = {
      assistant: "Claude Code",
      project: "C:/Users/Max/Desktop/ops",
      risk: "at-risk",
      export: "needs-export",
      control: "controllable"
    };

    const results = applySessionFilters(sessions, filters, {
      exportedSessionIds: new Set(["ses-001"])
    });

    expect(results.map((session) => session.sessionId)).toEqual(["ses-002"]);
  });

  it("can isolate sessions that are already ready to quarantine", () => {
    const results = applySessionFilters(
      buildSessions(),
      {
        assistant: "all",
        project: "all",
        risk: "all",
        export: "ready-to-quarantine",
        control: "all"
      },
      {
        exportedSessionIds: new Set(["ses-001"])
      }
    );

    expect(results.map((session) => session.sessionId)).toEqual(["ses-001"]);
  });
});

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
      todoItems: [],
      sessionControl: {
        supported: true,
        available: true,
        controller: "codex",
        command: "codex",
        attached: false
      }
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
      todoItems: [],
      sessionControl: {
        supported: true,
        available: true,
        controller: "claude-code",
        command: "claude",
        attached: true
      }
    },
    {
      sessionId: "ses-003",
      title: "Review cleanup handoff template",
      assistant: "OpenCode",
      progressState: "Completed",
      progressPercent: 100,
      lastActivityAt: "2026-03-13 08:00",
      environment: "Windows 11",
      valueScore: 70,
      summary: "Finalize the cleanup checklist wording.",
      projectPath: "C:/Users/Max/Desktop/docs",
      sourcePath: "C:/Users/Max/AppData/Local/OpenCode/ses-003.json",
      tags: ["docs"],
      riskFlags: [],
      keyArtifacts: ["cleanup-template.md"],
      transcriptHighlights: [],
      todoItems: []
    }
  ];
}
