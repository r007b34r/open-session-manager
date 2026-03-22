import type { SessionDetailRecord } from "./api";
import { buildRuleArtifact, buildSkillArtifact } from "./knowledge-lift";

describe("knowledge lift helpers", () => {
  it("builds a reusable rule artifact from session evidence", () => {
    const artifact = buildRuleArtifact(buildSession());

    expect(artifact).toContain("kind: osm-rule");
    expect(artifact).toContain("source_session: ses-002");
    expect(artifact).toContain("assistant: Claude Code");
    expect(artifact).toContain("## Open Tasks");
    expect(artifact).toContain("- Review shell hook chain");
    expect(artifact).toContain("- dangerous_permissions");
  });

  it("builds a reusable skill artifact with trigger, steps, and resume cue", () => {
    const artifact = buildSkillArtifact(buildSession());

    expect(artifact).toContain("name: audit-anthropic-relay-settings");
    expect(artifact).toContain("## Trigger");
    expect(artifact).toContain("relay");
    expect(artifact).toContain("## Steps");
    expect(artifact).toContain("## Resume Cue");
    expect(artifact).toContain("Mapped ANTHROPIC_BASE_URL override");
  });
});

function buildSession(): SessionDetailRecord {
  return {
    sessionId: "ses-002",
    title: "Audit Anthropic relay settings",
    assistant: "Claude Code",
    progressState: "Blocked",
    progressPercent: 15,
    lastActivityAt: "2026-03-14 22:10",
    environment: "Windows 11",
    valueScore: 47,
    summary:
      "Proxy endpoint and permissive shell hooks were identified, but remediation steps were not applied yet.",
    projectPath: "C:/Users/Max/Desktop/ops",
    sourcePath: "C:/Users/Max/.claude/projects/ops/claude-ses-1.jsonl",
    tags: ["relay", "risk", "claude"],
    riskFlags: ["dangerous_permissions", "shell_hook"],
    keyArtifacts: [
      "Documented ANTHROPIC_BASE_URL override path",
      "Captured hook command chain"
    ],
    transcriptHighlights: [
      {
        role: "Assistant",
        content:
          "Mapped ANTHROPIC_BASE_URL override and traced the permissive shell hook chain."
      }
    ],
    todoItems: [
      {
        content: "Review shell hook chain",
        completed: false
      },
      {
        content: "Export remediation summary before cleanup",
        completed: true
      }
    ]
  };
}
