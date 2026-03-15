import { afterEach, describe, expect, it, vi } from "vitest";

import { fetchDashboardSnapshot, type DashboardSnapshot } from "./api";

describe("fetchDashboardSnapshot", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
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
});
