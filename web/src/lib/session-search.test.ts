import { describe, expect, it } from "vitest";

import type { SessionDetailRecord } from "./api";
import { searchSessions } from "./session-search";

describe("searchSessions", () => {
  it("按字段权重排序命中结果", () => {
    const sessions = [
      buildSession("s1", {
        title: "Audit relay shell hook",
        summary: "Inspect Anthropic relay settings.",
        transcriptHighlights: [
          {
            role: "Assistant",
            content: "Mapped relay shell hook chain and captured the override."
          }
        ]
      }),
      buildSession("s2", {
        title: "General cleanup",
        summary: "Relay notes exist but the hook detail is buried.",
        keyArtifacts: ["relay hook appendix"]
      })
    ];

    const results = searchSessions(sessions, "relay hook");

    expect(results).toHaveLength(2);
    expect(results[0]?.session.sessionId).toBe("s1");
    expect(results[0]?.matchReasons).toContain("title");
    expect(results[0]?.snippet?.toLowerCase()).toContain("relay shell hook");
  });

  it("支持带引号的短语查询并输出上下文片段", () => {
    const sessions = [
      buildSession("s1", {
        title: "Prompt governance",
        transcriptHighlights: [
          {
            role: "Assistant",
            content: "The shell hook override still points at the old relay."
          }
        ]
      }),
      buildSession("s2", {
        title: "Prompt governance follow-up",
        transcriptHighlights: [
          {
            role: "Assistant",
            content: "The shell and the hook are handled in different stages."
          }
        ]
      })
    ];

    const results = searchSessions(sessions, "\"shell hook\"");

    expect(results).toHaveLength(1);
    expect(results[0]?.session.sessionId).toBe("s1");
    expect(results[0]?.snippet?.toLowerCase()).toContain("shell hook");
  });

  it("要求所有查询词都要命中后才返回结果", () => {
    const sessions = [
      buildSession("s1", {
        title: "Restore quarantine bundle",
        todoItems: [{ content: "Finalize manifest framing", completed: false }]
      }),
      buildSession("s2", {
        title: "Restore quarantine bundle",
        todoItems: [{ content: "Verify restore path", completed: false }]
      })
    ];

    const results = searchSessions(sessions, "manifest restore");

    expect(results).toHaveLength(1);
    expect(results[0]?.session.sessionId).toBe("s1");
    expect(results[0]?.matchReasons).toEqual(["title", "todo"]);
  });

  it("优先保留标题中的精确短语命中，不让低信号字段堆分数反超", () => {
    const sessions = [
      buildSession("s1", {
        title: "Relay cleanup",
        summary: "Tight title match for the cleanup queue."
      }),
      buildSession("s2", {
        title: "General notes",
        summary: "relay cleanup relay cleanup relay cleanup",
        keyArtifacts: ["relay cleanup appendix"],
        transcriptHighlights: [
          {
            role: "Assistant",
            content: "relay cleanup was mentioned again in a long transcript block."
          }
        ],
        todoItems: [{ content: "relay cleanup follow-up", completed: false }]
      })
    ];

    const results = searchSessions(sessions, "\"relay cleanup\"");

    expect(results).toHaveLength(2);
    expect(results[0]?.session.sessionId).toBe("s1");
    expect(results[0]?.matchReasons).toContain("title");
  });

  it("为 transcript 命中保留结构化定位信息，供详情页高亮对应条目", () => {
    const sessions = [
      buildSession("s1", {
        title: "Relay cleanup",
        transcriptHighlights: [
          {
            role: "User",
            content: "Review the current cleanup flow."
          },
          {
            role: "Assistant",
            content: "The shell hook override still points at the old relay endpoint."
          }
        ]
      })
    ];

    const results = searchSessions(sessions, "\"old relay\"");

    expect(results).toHaveLength(1);
    expect(results[0]?.matchReasons).toEqual(["transcript"]);
    expect(results[0]?.focus).toEqual({
      kind: "transcript",
      highlightIndex: 1,
      terms: ["old relay"]
    });
  });
});

function buildSession(
  sessionId: string,
  overrides: Partial<SessionDetailRecord> = {}
): SessionDetailRecord {
  return {
    sessionId,
    title: "Session title",
    assistant: "Codex",
    progressState: "In Progress",
    progressPercent: 50,
    lastActivityAt: "2026-03-15T10:00:00Z",
    environment: "Windows 11",
    valueScore: 50,
    summary: "Default summary",
    projectPath: "C:/Projects/demo",
    sourcePath: `C:/Users/Max/.codex/sessions/${sessionId}.jsonl`,
    tags: [],
    riskFlags: [],
    keyArtifacts: [],
    transcriptHighlights: [],
    todoItems: [],
    ...overrides
  };
}
