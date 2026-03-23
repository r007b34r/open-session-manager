import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");

test("web package exposes a dedicated visual regression script", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "web", "package.json"), "utf8")
  );

  assert.equal(
    packageJson.scripts["e2e:visual"],
    "cross-env NODE_PATH=./node_modules playwright test ../tests/e2e/open-session-manager-visual.spec.ts --project=chromium"
  );
});

test("visual regression spec exists in the e2e suite", () => {
  assert.equal(
    existsSync(path.join(repoRoot, "tests", "e2e", "open-session-manager-visual.spec.ts")),
    true
  );
});
