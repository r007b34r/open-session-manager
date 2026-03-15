import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");
const fixtureCatalogPath = path.join(__dirname, "fixtures", "catalog.json");
const realCatalogPath = path.join(repoRoot, "third_party", "upstreams", "catalog.json");
const libraryUrl = pathToFileURL(
  path.join(repoRoot, "scripts", "lib", "upstream-intake.mjs")
).href;
const cliPath = path.join(repoRoot, "scripts", "intake-upstreams.mjs");

test("loadUpstreamCatalog validates the committed governance catalog", async () => {
  const { loadUpstreamCatalog } = await import(libraryUrl);
  const catalog = await loadUpstreamCatalog(realCatalogPath);
  const agentSessions = catalog.entries.find(
    (entry) => entry.repo === "jazzyalex/agent-sessions"
  );
  const claudeCodeViewer = catalog.entries.find(
    (entry) => entry.repo === "d-kimuson/claude-code-viewer"
  );
  const claudeCodeLog = catalog.entries.find(
    (entry) => entry.repo === "daaain/claude-code-log"
  );
  const claudeCodeTools = catalog.entries.find(
    (entry) => entry.repo === "ChristopherA/claude_code_tools"
  );
  const broadSearch = catalog.entries.find(
    (entry) => entry.repo === "Dicklesworthstone/coding_agent_session_search"
  );
  const geminiCliWeb = catalog.entries.find(
    (entry) => entry.repo === "ssdeanx/Gemini-CLI-Web"
  );

  assert.ok(catalog.entries.length >= 6);
  assert.equal(agentSessions.absorption.mode, "candidate-absorb");
  assert.ok(
    agentSessions.verifiedPaths.includes("AgentSessions/AgentSessionsApp.swift")
  );
  assert.ok(
    agentSessions.upstreamSourceFiles.includes(
      "AgentSessions/Services/OpenClawSessionIndexer.swift"
    )
  );
  assert.ok(
    agentSessions.adoptedCapabilities.includes("OpenClaw session adapter")
  );
  assert.ok(
    claudeCodeViewer.adoptedCapabilities.includes(
      "Viewer-style transcript detail panel"
    )
  );
  assert.ok(
    claudeCodeViewer.upstreamSourceFiles.includes(
      "src/lib/todo-viewer/extractLatestTodos.ts"
    )
  );
  assert.ok(
    claudeCodeLog.adoptedCapabilities.includes("Transcript highlight digest")
  );
  assert.ok(
    claudeCodeLog.upstreamSourceFiles.includes(
      "claude_code_log/markdown/renderer.py"
    )
  );
  assert.equal(claudeCodeTools.absorption.mode, "candidate-absorb");
  assert.ok(
    claudeCodeTools.adoptedCapabilities.includes(
      "Markdown session handoff export"
    )
  );
  assert.equal(broadSearch.absorption.mode, "reference-only");
  assert.match(
    broadSearch.absorption.blockedBy.join(" "),
    /license/i
  );
  assert.equal(geminiCliWeb.absorption.mode, "reference-only");
  assert.match(geminiCliWeb.license.spdx, /Conflicting/);
});

test("research and attribution builders emit deterministic governance output", async () => {
  const {
    buildOpenSourceAttribution,
    buildUpstreamResearchIndex,
    buildUpstreamResearchReport,
    loadUpstreamCatalog
  } = await import(libraryUrl);
  const catalog = await loadUpstreamCatalog(fixtureCatalogPath);
  const claudeCodeLogReport = buildUpstreamResearchReport(
    catalog.entries.find((entry) => entry.repo === "daaain/claude-code-log")
  );
  const claudeCodeViewerReport = buildUpstreamResearchReport(
    catalog.entries.find((entry) => entry.repo === "d-kimuson/claude-code-viewer")
  );
  const index = buildUpstreamResearchIndex(catalog.entries);
  const attribution = buildOpenSourceAttribution(catalog.entries);

  assert.match(claudeCodeLogReport, /daaain\/claude-code-log/);
  assert.match(claudeCodeLogReport, /claude_code_log\/markdown\/renderer\.py/);
  assert.match(claudeCodeLogReport, /candidate-absorb/);
  assert.match(claudeCodeLogReport, /Transcript highlight digest/);
  assert.match(claudeCodeViewerReport, /d-kimuson\/claude-code-viewer/);
  assert.match(
    claudeCodeViewerReport,
    /src\/lib\/todo-viewer\/extractLatestTodos\.ts/
  );
  assert.match(
    claudeCodeViewerReport,
    /Viewer-style transcript detail panel/
  );
  assert.match(index, /reference-only/);
  assert.match(index, /LicenseRef-Restricted/);
  assert.match(attribution, /open Session Manager/);
  assert.match(attribution, /Viewer-style transcript detail panel/);
  assert.match(attribution, /Transcript highlight digest/);
  assert.match(attribution, /Do not copy code/i);
});

test("intake CLI can dry-run and materialize generated artifacts", () => {
  const tempRepoRoot = mkdtempSync(
    path.join(process.env.TEMP ?? process.cwd(), "osm-upstream-intake-")
  );

  const dryRunOutput = execFileSync(
    process.execPath,
    [
      cliPath,
      "--catalog",
      fixtureCatalogPath,
      "--repo-root",
      tempRepoRoot,
      "--dry-run"
    ],
    {
      cwd: repoRoot,
      encoding: "utf8"
    }
  );

  assert.match(dryRunOutput, /DRY RUN/);
  assert.match(dryRunOutput, /docs[\\/]+research[\\/]+upstreams[\\/]+index\.md/);
  assert.match(
    dryRunOutput,
    /docs[\\/]+release[\\/]+open-source-attribution\.md/
  );

  const writeOutput = execFileSync(
    process.execPath,
    [
      cliPath,
      "--catalog",
      fixtureCatalogPath,
      "--repo-root",
      tempRepoRoot
    ],
    {
      cwd: repoRoot,
      encoding: "utf8"
    }
  );

  assert.match(writeOutput, /WROTE/);

  const reportPath = path.join(
    tempRepoRoot,
    "docs",
    "research",
    "upstreams",
    "jazzyalex-agent-sessions.md"
  );
  const attributionPath = path.join(
    tempRepoRoot,
    "docs",
    "release",
    "open-source-attribution.md"
  );

  assert.equal(existsSync(reportPath), true);
  assert.equal(existsSync(attributionPath), true);
  assert.match(readFileSync(reportPath, "utf8"), /MIT/);
  assert.match(readFileSync(attributionPath, "utf8"), /reference-only/);
});
