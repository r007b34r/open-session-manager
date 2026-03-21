import assert from "node:assert/strict";
import test from "node:test";

import {
  diffFixtureSnapshots,
  normalizeFixtureSnapshot
} from "../../scripts/lib/fixture-snapshot.mjs";

test("normalizeFixtureSnapshot 会抹平运行时路径，避免 golden 基线依赖本机目录", () => {
  const normalized = normalizeFixtureSnapshot({
    sessions: [],
    configs: [],
    doctorFindings: [],
    auditEvents: [],
    metrics: [],
    usageOverview: {
      totals: {
        sessionsWithUsage: 0,
        inputTokens: 0,
        outputTokens: 0,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        reasoningTokens: 0,
        totalTokens: 0
      },
      assistants: []
    },
    runtime: {
      auditDbPath: "C:/Users/Max/AppData/Local/OpenSessionManager/audit/audit.db",
      exportRoot: "D:/OSM/exports",
      defaultExportRoot: "D:/OSM/exports",
      exportRootSource: "custom",
      quarantineRoot: "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine",
      preferencesPath: "C:/Users/Max/AppData/Local/OpenSessionManager/preferences.json"
    }
  });

  assert.equal(normalized.runtime.auditDbPath, "<runtime>");
  assert.equal(normalized.runtime.exportRoot, "<runtime>");
  assert.equal(normalized.runtime.defaultExportRoot, "<runtime>");
  assert.equal(normalized.runtime.quarantineRoot, "<runtime>");
  assert.equal(normalized.runtime.preferencesPath, "<runtime>");
  assert.equal(normalized.runtime.exportRootSource, "custom");
});

test("diffFixtureSnapshots 会返回发生漂移的具体 JSON 路径", () => {
  const differences = diffFixtureSnapshots(
    {
      sessions: [
        {
          sessionId: "ses-001",
          title: "old title"
        }
      ]
    },
    {
      sessions: [
        {
          sessionId: "ses-001",
          title: "new title"
        }
      ]
    }
  );

  assert.equal(differences.length, 1);
  assert.equal(differences[0].path, "$.sessions[0].title");
  assert.equal(differences[0].expected, "old title");
  assert.equal(differences[0].actual, "new title");
});
