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
