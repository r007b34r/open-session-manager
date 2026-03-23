import { afterEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import {
  applyConfigWriteback,
  applyGitProjectBranchSwitch,
  applyGitProjectCommit,
  applyGitProjectPush,
  applyMarkdownExport,
  recordSessionContinue,
  applySoftDelete,
  fetchDashboardSnapshot,
  type DashboardSnapshot,
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
        { label: "indexed_sessions", value: "1", note: "fixture_note" },
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
          keyArtifacts: ["artifact"],
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
          keyArtifacts: ["artifact"],
        },
      ],
      configs: [],
      doctorFindings: [],
      auditEvents: [],
      runtime: buildRuntime(),
    };

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => realSnapshot,
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(fetchDashboardSnapshot()).resolves.toEqual({
      ...realSnapshot,
      gitProjects: [],
      sessions: [
        {
          ...realSnapshot.sessions[1],
          transcriptHighlights: [],
          todoItems: [],
        },
        {
          ...realSnapshot.sessions[0],
          transcriptHighlights: [],
          todoItems: [],
        },
      ],
      usageOverview: buildEmptyUsageOverview(),
      usageTimeline: [],
    } satisfies DashboardSnapshot);
    expect(fetchMock).toHaveBeenCalledWith("/dashboard-snapshot.json", {
      cache: "no-store",
    });
  });

  it("保留审计事件里的 resume artifact 路径", async () => {
    const realSnapshot = {
      metrics: [],
      sessions: [],
      configs: [],
      doctorFindings: [],
      auditEvents: [
        {
          eventId: "evt-resume-artifact",
          type: "restore",
          target: "ses-001",
          actor: "r007b34r",
          createdAt: "2026-03-23T05:00:00Z",
          result: "success",
          detail: "Restored session.",
          resumeArtifactPath: "C:/OSM/exports/resume-ses-001.json",
        },
      ],
      runtime: buildRuntime(),
    };

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => realSnapshot,
    });
    vi.stubGlobal("fetch", fetchMock);

    const snapshot = await fetchDashboardSnapshot();

    expect(snapshot.auditEvents[0]).toMatchObject({
      resumeArtifactPath: "C:/OSM/exports/resume-ses-001.json",
    });
  });

  it("浏览器模式下真实快照不可用时默认不展示内置 fixture", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));

    const snapshot = await fetchDashboardSnapshot();

    expect(snapshot.sessions).toHaveLength(0);
    expect(snapshot.configs).toHaveLength(0);
    expect(snapshot.auditEvents).toHaveLength(0);
    expect(snapshot.usageOverview).toEqual(buildEmptyUsageOverview());
    expect(snapshot.runtime.auditDbPath).toBe("");
    expect(snapshot.runtime.quarantineRoot).toBe("");
    expect(snapshot.runtime.preferencesPath).toBe("");
  });

  it("只有显式开启 demo 模式时才允许回退到内置 fixture", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");

    const snapshot = await fetchDashboardSnapshot();

    expect(snapshot.sessions).toHaveLength(3);
    expect(snapshot.configs.map((config) => config.assistant)).toEqual(
      expect.arrayContaining(["GitHub Copilot CLI", "Factory Droid"]),
    );
  });

  it("桌面模式下优先调用 Tauri 原生命令读取快照", async () => {
    const nativeSnapshot: DashboardSnapshot = {
      metrics: [
        { label: "indexed_sessions", value: "2", note: "native_snapshot" },
      ],
      sessions: [],
      configs: [],
      doctorFindings: [],
      auditEvents: [],
      runtime: buildRuntime(),
    };
    const fetchMock = vi.fn();

    vi.stubGlobal("fetch", fetchMock);
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(fetchDashboardSnapshot()).resolves.toEqual({
      ...nativeSnapshot,
      gitProjects: [],
      usageOverview: buildEmptyUsageOverview(),
      usageTimeline: [],
    });
    expect(invokeMock).toHaveBeenCalledWith("load_dashboard_snapshot", {});
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("保留未知成本的 usage 记录，并把不可靠汇总成本标记为未知", async () => {
    const realSnapshot = {
      metrics: [],
      sessions: [
        {
          sessionId: "real-usage-unknown",
          title: "Unknown cost session",
          assistant: "Codex",
          progressState: "completed",
          progressPercent: 100,
          lastActivityAt: "2026-03-15T10:00:00Z",
          environment: "windows",
          valueScore: 95,
          summary: "Usage exists, but the upstream format did not expose cost.",
          projectPath: "C:/Projects/demo",
          sourcePath: "C:/Users/Max/.codex/sessions/demo.jsonl",
          tags: ["real"],
          riskFlags: [],
          keyArtifacts: ["artifact"],
          usage: {
            model: "gpt-5-codex",
            inputTokens: 120,
            outputTokens: 80,
            cacheReadTokens: 40,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 240,
            costSource: "unknown",
          },
        },
        {
          sessionId: "real-usage-zero",
          title: "Known zero cost session",
          assistant: "OpenCode",
          progressState: "completed",
          progressPercent: 100,
          lastActivityAt: "2026-03-15T09:00:00Z",
          environment: "linux",
          valueScore: 85,
          summary:
            "Usage exists and the provider explicitly reported zero cost.",
          projectPath: "/home/max/demo",
          sourcePath: "/home/max/.local/share/opencode/demo.json",
          tags: ["real"],
          riskFlags: [],
          keyArtifacts: ["artifact"],
          usage: {
            model: "gpt-5",
            inputTokens: 30,
            outputTokens: 10,
            cacheReadTokens: 0,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 40,
            costUsd: 0,
            costSource: "reported",
          },
        },
      ],
      configs: [],
      doctorFindings: [],
      auditEvents: [],
      usageTimeline: [
        {
          date: "2026-03-15",
          sessionsWithUsage: 2,
          inputTokens: 150,
          outputTokens: 90,
          cacheReadTokens: 40,
          cacheWriteTokens: 0,
          reasoningTokens: 0,
          totalTokens: 280,
          costSource: "unknown",
        },
      ],
      runtime: buildRuntime(),
    };

    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => realSnapshot,
      }),
    );

    const snapshot = await fetchDashboardSnapshot();
    const unknownCostSession = snapshot.sessions.find(
      (session) => session.sessionId === "real-usage-unknown",
    );
    const zeroCostSession = snapshot.sessions.find(
      (session) => session.sessionId === "real-usage-zero",
    );
    const codexUsage = snapshot.usageOverview.assistants.find(
      (assistant) => assistant.assistant === "Codex",
    );
    const openCodeUsage = snapshot.usageOverview.assistants.find(
      (assistant) => assistant.assistant === "OpenCode",
    );

    expect(unknownCostSession?.usage).toMatchObject({
      model: "gpt-5-codex",
      totalTokens: 240,
    });
    expect(unknownCostSession?.usage?.costUsd).toBeUndefined();
    expect((unknownCostSession?.usage as any)?.costSource).toBe("unknown");
    expect(zeroCostSession?.usage?.costUsd).toBe(0);
    expect((zeroCostSession?.usage as any)?.costSource).toBe("reported");
    expect(snapshot.usageOverview.totals.sessionsWithUsage).toBe(2);
    expect(snapshot.usageOverview.totals.totalTokens).toBe(280);
    expect(snapshot.usageOverview.totals.costUsd).toBeUndefined();
    expect((snapshot.usageOverview.totals as any).costSource).toBe("unknown");
    expect(codexUsage?.costUsd).toBeUndefined();
    expect((codexUsage as any)?.costSource).toBe("unknown");
    expect(openCodeUsage?.costUsd).toBe(0);
    expect((openCodeUsage as any)?.costSource).toBe("reported");
    expect((snapshot as any).usageTimeline).toEqual([
      expect.objectContaining({
        date: "2026-03-15",
        sessionsWithUsage: 2,
        totalTokens: 280,
        costUsd: undefined,
        costSource: "unknown",
      }),
    ]);
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
      "C:/Users/Max/AppData/Local/OpenSessionManager/preferences.json",
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
      costUsd: undefined,
      costSource: "unknown",
    },
    assistants: [],
  };
}

describe("desktop actions", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    invokeMock.mockReset();
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
    window.localStorage.removeItem("open-session-manager.enable-demo-data");
  });

  it("桌面模式下导出动作优先走 Tauri 命令", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
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
          detail: "Exported from native runtime.",
        },
        ...current.auditEvents,
      ],
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(applyMarkdownExport(current, "ses-001")).resolves.toEqual(
      nativeSnapshot,
    );
    expect(invokeMock).toHaveBeenCalledWith("export_session_markdown", {
      sessionId: "ses-001",
    });
  });

  it("桌面模式下隔离动作优先走 Tauri 命令", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      sessions: current.sessions.filter(
        (session) => session.sessionId !== "ses-003",
      ),
      auditEvents: [
        {
          eventId: "evt-native-delete",
          type: "soft_delete",
          target: "ses-003",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:31",
          result: "success",
          detail: "Deleted from native runtime.",
        },
        ...current.auditEvents,
      ],
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(applySoftDelete(current, "ses-003")).resolves.toEqual(
      nativeSnapshot,
    );
    expect(invokeMock).toHaveBeenCalledWith("soft_delete_session", {
      sessionId: "ses-003",
    });
  });

  it("桌面模式下配置写回动作优先走 Tauri 命令", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      configs: current.configs.map((config) =>
        config.artifactId === "cfg-004"
          ? {
              ...config,
              model: "gpt-5-mini",
              baseUrl: "https://github.com/api/copilot",
              maskedSecret: "***4321",
              officialOrProxy: "Official",
              risks: ["dangerous_permissions"],
            }
          : config,
      ),
      auditEvents: [
        {
          eventId: "evt-native-config-writeback",
          type: "config_writeback",
          target: "cfg-004",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:32",
          result: "success",
          detail: "Updated config from native runtime.",
        },
        ...current.auditEvents,
      ],
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(
      applyConfigWriteback(current, {
        artifactId: "cfg-004",
        assistant: "GitHub Copilot CLI",
        scope: "Global",
        path: "~/.copilot/config.json",
        provider: "github",
        model: "gpt-5-mini",
        baseUrl: "https://github.com/api/copilot",
        secret: "ghu_new_secret_123454321",
      }),
    ).resolves.toEqual(nativeSnapshot);
    expect(invokeMock).toHaveBeenCalledWith("write_config_artifact", {
      artifactId: "cfg-004",
      assistant: "GitHub Copilot CLI",
      scope: "Global",
      path: "~/.copilot/config.json",
      provider: "github",
      model: "gpt-5-mini",
      baseUrl: "https://github.com/api/copilot",
      secret: "ghu_new_secret_123454321",
    });
  });

  it("桌面模式下 Git 提交动作优先走 Tauri 命令", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      auditEvents: [
        {
          eventId: "evt-native-git-commit",
          type: "git_commit",
          target: "C:/Users/Max/Desktop/2026年3月15日",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:33",
          result: "success",
          detail: "Committed from native runtime.",
        },
        ...current.auditEvents,
      ],
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(
      applyGitProjectCommit(current, {
        repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
        message: "feat: native git commit",
      }),
    ).resolves.toEqual(nativeSnapshot);
    expect(invokeMock).toHaveBeenCalledWith("commit_git_project", {
      repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
      message: "feat: native git commit",
    });
  });

  it("桌面模式下 Git 切分支动作优先走 Tauri 命令", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      auditEvents: [
        {
          eventId: "evt-native-git-switch",
          type: "git_branch_switch",
          target: "C:/Users/Max/Desktop/2026年3月15日",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:34",
          result: "success",
          detail: "Switched from native runtime.",
        },
        ...current.auditEvents,
      ],
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(
      applyGitProjectBranchSwitch(current, {
        repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
        branch: "feature/native-branch",
      }),
    ).resolves.toEqual(nativeSnapshot);
    expect(invokeMock).toHaveBeenCalledWith("switch_git_project_branch", {
      repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
      branch: "feature/native-branch",
    });
  });

  it("桌面模式下 Git 推送动作优先走 Tauri 命令", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const nativeSnapshot: DashboardSnapshot = {
      ...current,
      auditEvents: [
        {
          eventId: "evt-native-git-push",
          type: "git_push",
          target: "C:/Users/Max/Desktop/2026年3月15日",
          actor: "r007b34r",
          createdAt: "2026-03-15 15:35",
          result: "success",
          detail: "Pushed from native runtime.",
        },
        ...current.auditEvents,
      ],
    };

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
    invokeMock.mockResolvedValueOnce(nativeSnapshot);

    await expect(
      applyGitProjectPush(current, {
        repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
      }),
    ).resolves.toEqual(nativeSnapshot);
    expect(invokeMock).toHaveBeenCalledWith("push_git_project", {
      repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
      remote: undefined,
    });
  });
});

describe("session control fallback", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    invokeMock.mockReset();
    Reflect.deleteProperty(window, "__TAURI_INTERNALS__");
    window.localStorage.removeItem("open-session-manager.enable-demo-data");
  });

  it("detached 会话不能直接继续运行", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();

    const nextSnapshot = recordSessionContinue(current, {
      sessionId: "ses-001",
      prompt: "Continue while detached",
    });
    const target = nextSnapshot.sessions.find((session) => session.sessionId === "ses-001");

    expect(target?.sessionControl?.attached).toBe(false);
    expect(target?.sessionControl?.lastPrompt).toBeUndefined();
    expect(
      nextSnapshot.auditEvents.some((event) => event.type === "session_continue"),
    ).toBe(false);
  });

  it("busy 会话不能继续运行", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const busySnapshot = {
      ...current,
      sessions: current.sessions.map((session) =>
        session.sessionId === "ses-001"
          ? {
              ...session,
              sessionControl: {
                ...session.sessionControl,
                supported: true,
                available: true,
                attached: true,
                runtimeState: "busy" as const,
              },
            }
          : session,
      ),
    };

    const nextSnapshot = recordSessionContinue(busySnapshot, {
      sessionId: "ses-001",
      prompt: "Continue while busy",
    });

    expect(nextSnapshot).toEqual(busySnapshot);
  });

  it("冷却窗口内不会重复继续运行", async () => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    const current = await fetchDashboardSnapshot();
    const throttledSnapshot = {
      ...current,
      sessions: current.sessions.map((session) =>
        session.sessionId === "ses-001"
          ? {
              ...session,
              sessionControl: {
                ...session.sessionControl,
                supported: true,
                available: true,
                attached: true,
                runtimeState: "waiting" as const,
                lastContinuedAt: new Date().toISOString(),
              },
            }
          : session,
      ),
    };

    const nextSnapshot = recordSessionContinue(throttledSnapshot, {
      sessionId: "ses-001",
      prompt: "Continue too fast",
    });

    expect(nextSnapshot).toEqual(throttledSnapshot);
  });
});
