import crypto from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

export const FIXTURE_GROUPS = [
  {
    id: "codex-session",
    assistant: "codex",
    kind: "session",
    relativePath: "codex",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["jazzyalex/agent-sessions"]
  },
  {
    id: "claude-session",
    assistant: "claude-code",
    kind: "session",
    relativePath: "claude",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["daaain/claude-code-log", "d-kimuson/claude-code-viewer"]
  },
  {
    id: "opencode-session",
    assistant: "opencode",
    kind: "session",
    relativePath: "opencode",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["daaain/claude-code-log"]
  },
  {
    id: "gemini-session",
    assistant: "gemini-cli",
    kind: "session",
    relativePath: "gemini",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["jazzyalex/agent-sessions"]
  },
  {
    id: "copilot-session",
    assistant: "github-copilot-cli",
    kind: "session",
    relativePath: "copilot",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["jazzyalex/agent-sessions", "endorhq/rover"]
  },
  {
    id: "factory-session",
    assistant: "factory-droid",
    kind: "session",
    relativePath: "factory",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["jazzyalex/agent-sessions"]
  },
  {
    id: "openclaw-session",
    assistant: "openclaw",
    kind: "session",
    relativePath: "openclaw",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["jazzyalex/agent-sessions", "farion1231/cc-switch"]
  },
  {
    id: "config-artifacts",
    assistant: "multiple",
    kind: "config",
    relativePath: "configs",
    fixtureVersion: "2026-03-16",
    source: "clean-room synthetic",
    upstreams: ["farion1231/cc-switch", "endorhq/rover"]
  }
];

export async function generateFixtureLedger(fixturesRoot, groups = FIXTURE_GROUPS) {
  const normalizedRoot = path.resolve(fixturesRoot);
  const fixtures = [];

  for (const group of groups) {
    const groupRoot = path.join(normalizedRoot, group.relativePath);
    const files = await collectRelativeFiles(groupRoot);
    const entries = await Promise.all(
      files.map(async (relativePath) => {
        const absolutePath = path.join(groupRoot, relativePath);
        const content = await fs.readFile(absolutePath);
        const normalizedPath = normalizeRelativePath(
          path.join(group.relativePath, relativePath)
        );

        return {
          relativePath: normalizedPath,
          contentHash: hashBytes(content)
        };
      })
    );
    entries.sort((left, right) => left.relativePath.localeCompare(right.relativePath));

    fixtures.push({
      ...group,
      contentHash: hashText(
        entries.map((entry) => `${entry.relativePath}:${entry.contentHash}`).join("\n")
      ),
      fileCount: entries.length,
      files: entries
    });
  }

  return {
    version: 1,
    generatedAt: new Date().toISOString(),
    fixtures
  };
}

export async function detectFixtureLedgerDrift(
  baselineLedger,
  fixturesRoot,
  groups = FIXTURE_GROUPS
) {
  const currentLedger = await generateFixtureLedger(fixturesRoot, groups);
  const baselineFixtures = new Map(
    (baselineLedger.fixtures ?? []).map((fixture) => [fixture.id, fixture])
  );
  const currentFixtures = new Map(currentLedger.fixtures.map((fixture) => [fixture.id, fixture]));
  const addedFixtures = [];
  const removedFixtures = [];
  const changed = [];

  for (const fixtureId of currentFixtures.keys()) {
    if (!baselineFixtures.has(fixtureId)) {
      addedFixtures.push(fixtureId);
    }
  }

  for (const fixtureId of baselineFixtures.keys()) {
    if (!currentFixtures.has(fixtureId)) {
      removedFixtures.push(fixtureId);
    }
  }

  for (const [fixtureId, currentFixture] of currentFixtures) {
    const baselineFixture = baselineFixtures.get(fixtureId);
    if (!baselineFixture) {
      continue;
    }

    const baselineFiles = new Map(
      (baselineFixture.files ?? []).map((entry) => [entry.relativePath, entry.contentHash])
    );
    const currentFiles = new Map(
      currentFixture.files.map((entry) => [entry.relativePath, entry.contentHash])
    );
    const allPaths = new Set([...baselineFiles.keys(), ...currentFiles.keys()]);

    for (const relativePath of [...allPaths].sort()) {
      const expectedHash = baselineFiles.get(relativePath);
      const actualHash = currentFiles.get(relativePath);

      if (expectedHash === actualHash) {
        continue;
      }

      changed.push({
        fixtureId,
        relativePath,
        expectedHash,
        actualHash
      });
    }
  }

  return {
    status:
      addedFixtures.length > 0 || removedFixtures.length > 0 || changed.length > 0
        ? "drift"
        : "ok",
    addedFixtures,
    removedFixtures,
    changed,
    currentLedger
  };
}

export async function readFixtureLedger(ledgerPath) {
  const payload = JSON.parse(await fs.readFile(ledgerPath, "utf8"));
  return payload;
}

export async function writeFixtureLedger(ledgerPath, ledger) {
  await fs.mkdir(path.dirname(ledgerPath), { recursive: true });
  await fs.writeFile(ledgerPath, `${JSON.stringify(ledger, null, 2)}\n`);
}

async function collectRelativeFiles(root) {
  try {
    const stats = await fs.stat(root);
    if (!stats.isDirectory()) {
      return [];
    }
  } catch {
    return [];
  }

  const files = [];
  await visit(root, "");
  return files.sort();

  async function visit(currentRoot, relativePrefix) {
    const entries = await fs.readdir(currentRoot, { withFileTypes: true });

    for (const entry of entries) {
      const nextAbsolute = path.join(currentRoot, entry.name);
      const nextRelative = relativePrefix
        ? path.join(relativePrefix, entry.name)
        : entry.name;

      if (entry.isDirectory()) {
        await visit(nextAbsolute, nextRelative);
        continue;
      }

      if (entry.isFile()) {
        files.push(nextRelative);
      }
    }
  }
}

function normalizeRelativePath(value) {
  return value.split(path.sep).join("/");
}

function hashBytes(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function hashText(value) {
  return hashBytes(Buffer.from(value, "utf8"));
}
