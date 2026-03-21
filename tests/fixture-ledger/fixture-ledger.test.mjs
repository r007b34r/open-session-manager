import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  FIXTURE_GROUPS,
  detectFixtureLedgerDrift,
  generateFixtureLedger
} from "../../scripts/lib/fixture-ledger.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");

test("generateFixtureLedger 为已提交 fixtures 产出版本、来源和稳定 hash", async () => {
  const ledger = await generateFixtureLedger(path.join(repoRoot, "tests", "fixtures"));

  assert.equal(ledger.version, 1);
  assert.ok(Array.isArray(ledger.fixtures));
  assert.ok(ledger.fixtures.length >= 7);

  const codexFixture = ledger.fixtures.find((fixture) => fixture.id === "codex-session");
  assert.ok(codexFixture);
  assert.equal(codexFixture.assistant, "codex");
  assert.equal(codexFixture.kind, "session");
  assert.equal(codexFixture.fixtureVersion, "2026-03-16");
  assert.equal(codexFixture.source, "clean-room synthetic");
  assert.match(codexFixture.contentHash, /^[a-f0-9]{64}$/);
  assert.ok(codexFixture.fileCount >= 1);
});

test("detectFixtureLedgerDrift 能报告发生漂移的 fixture 文件", async () => {
  const tempRoot = await fs.mkdtemp(path.join(os.tmpdir(), "osm-fixture-ledger-"));
  const fixturesRoot = path.join(tempRoot, "fixtures");
  const codexRoot = path.join(fixturesRoot, "codex");
  const codexFile = path.join(codexRoot, "rollout-demo.jsonl");
  const groups = [
    {
      id: "codex-session",
      assistant: "codex",
      kind: "session",
      relativePath: "codex",
      fixtureVersion: "2026-03-16",
      source: "clean-room synthetic",
      upstreams: ["jazzyalex/agent-sessions"]
    }
  ];

  await fs.mkdir(codexRoot, { recursive: true });
  await fs.writeFile(codexFile, "{\"type\":\"session_meta\"}\n");

  const baselineLedger = await generateFixtureLedger(fixturesRoot, groups);

  await fs.writeFile(
    codexFile,
    "{\"type\":\"session_meta\"}\n{\"type\":\"response_item\"}\n"
  );

  const drift = await detectFixtureLedgerDrift(baselineLedger, fixturesRoot, groups);

  assert.equal(drift.status, "drift");
  assert.equal(drift.changed.length, 1);
  assert.equal(drift.changed[0].fixtureId, "codex-session");
  assert.equal(drift.changed[0].relativePath, "codex/rollout-demo.jsonl");
  assert.match(drift.changed[0].actualHash, /^[a-f0-9]{64}$/);
});

test("FIXTURE_GROUPS 覆盖当前公开支持的 fixture 目录", () => {
  const fixtureIds = new Set(FIXTURE_GROUPS.map((fixture) => fixture.id));

  assert.ok(fixtureIds.has("codex-session"));
  assert.ok(fixtureIds.has("claude-session"));
  assert.ok(fixtureIds.has("opencode-session"));
  assert.ok(fixtureIds.has("gemini-session"));
  assert.ok(fixtureIds.has("copilot-session"));
  assert.ok(fixtureIds.has("factory-session"));
  assert.ok(fixtureIds.has("openclaw-session"));
  assert.ok(fixtureIds.has("config-artifacts"));
});
