import { execFileSync } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const VALID_PHASES = new Set(["plan", "red", "green", "verify", "review", "docs"]);
export const NOTE_REF = "refs/notes/osm-tdd";

function ensureNonEmptyString(value, fieldName) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`Expected ${fieldName} to be a non-empty string.`);
  }

  return value.trim();
}

function ensureItemId(value) {
  const itemId = ensureNonEmptyString(value, "item");

  if (!/^[A-Z0-9]+-\d+$/.test(itemId)) {
    throw new Error(`Unsupported item id: ${itemId}`);
  }

  return itemId;
}

function ensurePhase(value) {
  const phase = ensureNonEmptyString(value, "phase").toLowerCase();

  if (!VALID_PHASES.has(phase)) {
    throw new Error(`Unsupported phase: ${phase}`);
  }

  return phase;
}

function ensureCommands(value) {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.map((command, index) =>
    ensureNonEmptyString(command, `commands[${index}]`)
  );
}

function runGit(repoRoot, args) {
  return execFileSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8"
  }).trimEnd();
}

function safeRunGit(repoRoot, args) {
  try {
    return runGit(repoRoot, args);
  } catch {
    return "";
  }
}

function resolveGitDirectory(repoRoot) {
  const gitDirectory = ensureNonEmptyString(
    safeRunGit(repoRoot, ["rev-parse", "--git-dir"]),
    "gitDirectory"
  );

  return path.isAbsolute(gitDirectory)
    ? gitDirectory
    : path.resolve(repoRoot, gitDirectory);
}

export function formatTimestamp(date = new Date()) {
  const parts = [
    String(date.getFullYear()),
    String(date.getMonth() + 1).padStart(2, "0"),
    String(date.getDate()).padStart(2, "0"),
    "-",
    String(date.getHours()).padStart(2, "0"),
    String(date.getMinutes()).padStart(2, "0"),
    String(date.getSeconds()).padStart(2, "0")
  ];

  return parts.join("");
}

export function normalizeCheckpointOptions(options) {
  const repoRoot = path.resolve(
    ensureNonEmptyString(options.repoRoot ?? process.cwd(), "repoRoot")
  );
  const itemId = ensureItemId(options.itemId);
  const phase = ensurePhase(options.phase);

  return {
    repoRoot,
    itemId,
    phase,
    note: options.note ? ensureNonEmptyString(options.note, "note") : "",
    commands: ensureCommands(options.commands),
    createdAt: options.createdAt instanceof Date ? options.createdAt : new Date(),
    outputPath: options.outputPath ? path.resolve(options.outputPath) : undefined,
    dryRun: options.dryRun === true
  };
}

export function collectGitState(repoRoot) {
  return {
    statusShort: safeRunGit(repoRoot, ["status", "--short", "--branch"]),
    diffStat: safeRunGit(repoRoot, ["diff", "--stat"]),
    lastCommit: safeRunGit(repoRoot, ["log", "-1", "--oneline"]),
    headSha: safeRunGit(repoRoot, ["rev-parse", "HEAD"])
  };
}

export function buildCheckpointTagName(itemId, phase, createdAt) {
  return `osm/tdd/${itemId}/${phase}/${formatTimestamp(createdAt)}`;
}

export function defaultReviewPath(repoRoot, itemId, phase, createdAt) {
  const gitDirectory = resolveGitDirectory(repoRoot);

  return path.join(
    gitDirectory,
    "osm",
    "reviews",
    `${formatTimestamp(createdAt)}-${itemId}-${phase}.md`
  );
}

export function buildReviewSnapshotMarkdown({
  itemId,
  phase,
  note,
  commands,
  createdAt,
  gitState
}) {
  const commandLines = commands
    .concat([
      "git status --short --branch",
      "git diff --stat",
      "git log -1 --oneline"
    ])
    .map((command) => `- ${command}`)
    .join("\n");

  return `# Git Review Snapshot

- Item: ${itemId}
- Phase: ${phase}
- Created At: ${createdAt.toISOString()}
- Note: ${note || "(none)"}

## Commands

${commandLines}

## Git Status

\`\`\`text
${gitState.statusShort || "(clean)"}
\`\`\`

## Diff Stat

\`\`\`text
${gitState.diffStat || "(no diff)"}
\`\`\`

## Last Commit

\`\`\`text
${gitState.lastCommit || "(no commit)"}
\`\`\`
`;
}

export async function writeReviewSnapshot(options) {
  const normalized = normalizeCheckpointOptions(options);
  const gitState = collectGitState(normalized.repoRoot);
  const reviewPath =
    normalized.outputPath ??
    defaultReviewPath(
      normalized.repoRoot,
      normalized.itemId,
      normalized.phase,
      normalized.createdAt
    );
  const content = buildReviewSnapshotMarkdown({
    itemId: normalized.itemId,
    phase: normalized.phase,
    note: normalized.note,
    commands: normalized.commands,
    createdAt: normalized.createdAt,
    gitState
  });

  await mkdir(path.dirname(reviewPath), { recursive: true });
  await writeFile(reviewPath, content, "utf8");

  return {
    reviewPath,
    content,
    gitState,
    createdAt: normalized.createdAt,
    itemId: normalized.itemId,
    phase: normalized.phase,
    note: normalized.note,
    commands: normalized.commands
  };
}

export function buildCheckpointNote({
  itemId,
  phase,
  note,
  commands,
  createdAt,
  reviewPath,
  gitState,
  tagName
}) {
  return `Item: ${itemId}
Phase: ${phase}
Created At: ${createdAt.toISOString()}
Tag: ${tagName}
Review: ${reviewPath}
Note: ${note || "(none)"}
Commands:
${commands.map((command) => `- ${command}`).join("\n") || "- (none)"}

Git Status:
${gitState.statusShort || "(clean)"}

Diff Stat:
${gitState.diffStat || "(no diff)"}

Last Commit:
${gitState.lastCommit || "(no commit)"}
`;
}

export async function recordCheckpoint(options) {
  const normalized = normalizeCheckpointOptions(options);
  const reviewPath =
    normalized.outputPath ??
    defaultReviewPath(
      normalized.repoRoot,
      normalized.itemId,
      normalized.phase,
      normalized.createdAt
    );
  const gitState = collectGitState(normalized.repoRoot);
  const tagName = buildCheckpointTagName(
    normalized.itemId,
    normalized.phase,
    normalized.createdAt
  );

  if (!normalized.dryRun) {
    await mkdir(path.dirname(reviewPath), { recursive: true });
    const reviewSnapshot = await writeReviewSnapshot({
      ...normalized,
      outputPath: reviewPath
    });
    const noteContent = buildCheckpointNote({
      itemId: normalized.itemId,
      phase: normalized.phase,
      note: normalized.note,
      commands: normalized.commands,
      createdAt: normalized.createdAt,
      reviewPath,
      gitState: reviewSnapshot.gitState,
      tagName
    });

    runGit(normalized.repoRoot, [
      "tag",
      "-a",
      tagName,
      "-m",
      `${normalized.itemId} ${normalized.phase}: ${normalized.note || "checkpoint"}`
    ]);
    runGit(normalized.repoRoot, [
      "notes",
      "--ref",
      NOTE_REF,
      "add",
      "-f",
      "-m",
      noteContent,
      "HEAD"
    ]);
  }

  return {
    dryRun: normalized.dryRun,
    noteRef: NOTE_REF,
    repoRoot: normalized.repoRoot,
    reviewPath,
    tagName,
    gitState,
    createdAt: normalized.createdAt,
    itemId: normalized.itemId,
    phase: normalized.phase,
    note: normalized.note,
    commands: normalized.commands
  };
}
