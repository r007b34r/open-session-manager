import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");

test("web package exposes a formal browser preview launch script", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "web", "package.json"), "utf8")
  );

  assert.equal(packageJson.scripts.preview, "vite preview");
  assert.equal(
    packageJson.scripts.browser,
    "npm run build && npm run preview -- --host 127.0.0.1 --port 4173 --strictPort"
  );
});

test("playwright e2e uses the browser preview script instead of the dev server", () => {
  const config = readFileSync(path.join(repoRoot, "web", "playwright.config.ts"), "utf8");

  assert.match(config, /command:\s*"npm run browser"/);
  assert.doesNotMatch(config, /npm run dev/);
});
