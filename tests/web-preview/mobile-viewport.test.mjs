import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");

test("playwright e2e covers both desktop and mobile viewports", () => {
  const config = readFileSync(path.join(repoRoot, "web", "playwright.config.ts"), "utf8");

  assert.match(config, /name:\s*"chromium"/);
  assert.match(config, /name:\s*"mobile-chrome"/);
  assert.match(config, /devices\["Pixel 7"\]/);
});
