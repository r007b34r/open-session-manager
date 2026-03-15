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
    const realSnapshot: DashboardSnapshot = {
      metrics: [
        { label: "indexed_sessions", value: "1", note: "fixture_note" }
      ],
      sessions: [
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
      auditEvents: []
    };

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => realSnapshot
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchDashboardSnapshot()).resolves.toEqual(realSnapshot);
    expect(fetchMock).toHaveBeenCalledWith("/dashboard-snapshot.json", {
      cache: "no-store"
    });
  });

  it("真实快照不可用时退回到内置 fixture", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));

    const snapshot = await fetchDashboardSnapshot();

    expect(snapshot.sessions).toHaveLength(3);
    expect(snapshot.configs).toHaveLength(3);
  });

  it("桌面模式下优先调用 Tauri 原生命令读取快照", async () => {
    const nativeSnapshot: DashboardSnapshot = {
      metrics: [
        { label: "indexed_sessions", value: "2", note: "native_snapshot" }
      ],
      sessions: [],
      configs: [],
      auditEvents: []
    };
    const fetchMock = vi.fn();

    vi.stubGlobal("fetch", fetchMock);
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {}
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(fetchDashboardSnapshot()).resolves.toEqual(nativeSnapshot);
    expect(invokeMock).toHaveBeenCalledWith("load_dashboard_snapshot", {});
    expect(fetchMock).not.toHaveBeenCalled();
  });
});

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
          actor: "Max",
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
          actor: "Max",
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
