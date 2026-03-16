import { afterEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

import {
  applyMarkdownExport,
  applySoftDelete,
  fetchDashboardSnapshot,
  type DashboardSnapshot
} from "./api";

describe("fetchDashboardSnapshot", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    invokeMock.mockReset();
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
  });

  it("优先读取真实快照接口", async () => {
    const realSnapshot = {
      metrics: [
        { label: "indexed_sessions", value: "1", note: "fixture_note" }
      ],
      sessions: [
        {
          sessionId: "real-older",
          title: "Older snapshot title",
          assistant: "Codex",
          progressState: "completed",
          progressPercent: 100,
          lastActivityAt: "2026-03-15T09:00:00Z",
          environment: "windows",
          valueScore: 75,
          summary: "Older snapshot session.",
          projectPath: "C:/Projects/demo",
          sourcePath: "C:/Users/Max/.codex/sessions/older.jsonl",
          tags: ["real"],
          riskFlags: [],
          keyArtifacts: ["artifact"]
        },
        {
          sessionId: "real-001",
          title: "Real snapshot title",
          assistant: "Codex",
          progressState: "completed",
          progressPercent: 100,
          lastActivityAt: "2026-03-15T10:00:00Z",
          environment: "windows",
          valueScore: 95,
          summary: "Loaded from a real local snapshot.",
          projectPath: "C:/Projects/demo",
          sourcePath: "C:/Users/Max/.codex/sessions/demo.jsonl",
          tags: ["real"],
          riskFlags: [],
          keyArtifacts: ["artifact"]
        }
      ],
      configs: [],
      auditEvents: [],
      runtime: buildRuntime()
    };

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => realSnapshot
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchDashboardSnapshot()).resolves.toEqual({
      ...realSnapshot,
      sessions: [
        {
          ...realSnapshot.sessions[1],
          transcriptHighlights: [],
          todoItems: [],
          usage: undefined
        },
        {
          ...realSnapshot.sessions[0],
          transcriptHighlights: [],
          todoItems: [],
          usage: undefined
        }
      ],
      usageOverview: buildEmptyUsageOverview()
    } satisfies DashboardSnapshot);
    expect(fetchMock).toHaveBeenCalledWith("/dashboard-snapshot.json", {
      cache: "no-store"
    });
  });

  it("真实快照不可用时退回到内置 fixture", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));

    const snapshot = await fetchDashboardSnapshot();

    expect(snapshot.sessions).toHaveLength(3);
    expect(snapshot.configs).toHaveLength(5);
    expect(snapshot.configs.map((config) => config.assistant)).toEqual(
      expect.arrayContaining(["GitHub Copilot CLI", "Factory Droid"])
    );
  });

  it("桌面模式下优先调用 Tauri 原生命令读取快照", async () => {
    const nativeSnapshot: DashboardSnapshot = {
      metrics: [
        { label: "indexed_sessions", value: "2", note: "native_snapshot" }
      ],
      sessions: [],
      configs: [],
      auditEvents: [],
      runtime: buildRuntime()
    };
    const fetchMock = vi.fn();

    vi.stubGlobal("fetch", fetchMock);
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {}
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(fetchDashboardSnapshot()).resolves.toEqual({
      ...nativeSnapshot,
      usageOverview: buildEmptyUsageOverview()
    });
    expect(invokeMock).toHaveBeenCalledWith("load_dashboard_snapshot", {});
    expect(fetchMock).not.toHaveBeenCalled();
  });
});

function buildRuntime() {
  return {
    auditDbPath: "C:/Users/Max/AppData/Local/OpenSessionManager/audit/audit.db",
    exportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
    defaultExportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
    exportRootSource: "default" as const,
    quarantineRoot: "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine",
    preferencesPath:
      "C:/Users/Max/AppData/Local/OpenSessionManager/preferences.json"
  };
}

function buildEmptyUsageOverview() {
  return {
    totals: {
      sessionsWithUsage: 0,
      inputTokens: 0,
      outputTokens: 0,
      cacheReadTokens: 0,
      cacheWriteTokens: 0,
      reasoningTokens: 0,
      totalTokens: 0,
      costUsd: 0
    },
    assistants: []
  };
}

describe("desktop actions", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    invokeMock.mockReset();
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
  });

  it("桌面模式下导出动作优先走 Tauri 命令", async () => {
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      auditEvents: [
        {
          eventId: "evt-native-export",
          type: "export_markdown",
          target: "ses-001",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:30",
          result: "success",
          detail: "Exported from native runtime."
        },
        ...current.auditEvents
      ]
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {}
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(applyMarkdownExport(current, "ses-001")).resolves.toEqual(
      nativeSnapshot
    );
    expect(invokeMock).toHaveBeenCalledWith("export_session_markdown", {
      sessionId: "ses-001"
    });
  });

  it("桌面模式下隔离动作优先走 Tauri 命令", async () => {
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      sessions: current.sessions.filter((session) => session.sessionId !== "ses-003"),
      auditEvents: [
        {
          eventId: "evt-native-delete",
          type: "soft_delete",
          target: "ses-003",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:31",
          result: "success",
          detail: "Deleted from native runtime."
        },
        ...current.auditEvents
      ]
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {}
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(applySoftDelete(current, "ses-003")).resolves.toEqual(
      nativeSnapshot
    );
    expect(invokeMock).toHaveBeenCalledWith("soft_delete_session", {
      sessionId: "ses-003"
    });
  });
});
