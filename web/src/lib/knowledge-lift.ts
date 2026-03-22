import type { SessionDetailRecord } from "./api";

type ArtifactMode = "rule" | "skill";

export function buildRuleArtifact(session: SessionDetailRecord) {
  return buildArtifact(session, "rule");
}

export function buildSkillArtifact(session: SessionDetailRecord) {
  return buildArtifact(session, "skill");
}

function buildArtifact(session: SessionDetailRecord, mode: ArtifactMode) {
  const header =
    mode === "rule"
      ? [
          "---",
          "kind: osm-rule",
          `name: ${buildArtifactName(session.title)}`,
          `source_session: ${session.sessionId}`,
          `assistant: ${session.assistant}`,
          `status: ${session.progressState}`,
          "---"
        ]
      : [
          "---",
          "kind: osm-skill",
          `name: ${buildArtifactName(session.title)}`,
          `source_session: ${session.sessionId}`,
          `assistant: ${session.assistant}`,
          "---"
        ];

  const body =
    mode === "rule"
      ? [
          `# ${session.title}`,
          "",
          "## Summary",
          session.summary,
          "",
          "## Scope",
          `- Project: ${session.projectPath}`,
          `- Source: ${session.sourcePath}`,
          `- Tags: ${formatInlineList(session.tags)}`,
          "",
          "## Open Tasks",
          ...toBulletLines(
            session.todoItems.filter((item) => !item.completed).map((item) => item.content),
            "No open tasks captured."
          ),
          "",
          "## Completed Tasks",
          ...toBulletLines(
            session.todoItems.filter((item) => item.completed).map((item) => item.content),
            "No completed tasks captured."
          ),
          "",
          "## Risk Flags",
          ...toBulletLines(session.riskFlags, "No risk flags captured."),
          "",
          "## Evidence",
          ...toBulletLines(session.keyArtifacts, "No supporting evidence captured.")
        ]
      : [
          `# ${session.title}`,
          "",
          "## Trigger",
          `Use this skill when working on ${session.assistant} sessions that match ${formatInlineList(session.tags)}.`,
          session.summary,
          "",
          "## Steps",
          ...toNumberedLines(buildSkillSteps(session)),
          "",
          "## Resume Cue",
          buildResumeCue(session)
        ];

  return [...header, "", ...body].join("\n");
}

function buildSkillSteps(session: SessionDetailRecord) {
  const steps = [
    ...session.todoItems
      .filter((item) => !item.completed)
      .map((item) => item.content),
    ...buildEvidenceLines(session),
    ...session.riskFlags.map((flag) => `Review ${flag}`)
  ];

  return [...new Set(steps)].slice(0, 6);
}

function buildResumeCue(session: SessionDetailRecord) {
  const openTasks = session.todoItems
    .filter((item) => !item.completed)
    .map((item) => item.content);
  const transcriptCue = session.transcriptHighlights[0]?.content;
  const riskCue = session.riskFlags.length > 0 ? session.riskFlags.join(", ") : "none";

  return [
    `Resume from ${session.sessionId} (${session.assistant}) at ${session.lastActivityAt}.`,
    `Outstanding tasks: ${openTasks.length > 0 ? openTasks.join("; ") : "none"}.`,
    `Risk focus: ${riskCue}.`,
    transcriptCue ? `Last useful transcript cue: ${transcriptCue}` : null
  ]
    .filter((value): value is string => Boolean(value))
    .join(" ");
}

function buildEvidenceLines(session: SessionDetailRecord) {
  return [
    ...session.keyArtifacts,
    ...session.transcriptHighlights.map((highlight) => highlight.content)
  ];
}

function toBulletLines(lines: string[], emptyLine: string) {
  if (lines.length === 0) {
    return [`- ${emptyLine}`];
  }

  return lines.map((line) => `- ${line}`);
}

function toNumberedLines(lines: string[]) {
  if (lines.length === 0) {
    return ["1. Review the exported session evidence."];
  }

  return lines.map((line, index) => `${index + 1}. ${line}`);
}

function buildArtifactName(title: string) {
  const normalized = title
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return normalized || "session-knowledge-lift";
}

function formatInlineList(values: string[]) {
  return values.length > 0 ? values.join(", ") : "no tags";
}
