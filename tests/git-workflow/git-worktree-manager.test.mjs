import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");
const worktreeCliPath = path.join(repoRoot, "scripts", "git-worktree-manager.mjs");

function createTempGitRepo() {
  const tempRoot = mkdtempSync(
    path.join(process.env.TEMP ?? process.cwd(), "osm-git-worktree-manager-")
  );

  execFileSync("git", ["init", "-b", "main"], {
    cwd: tempRoot,
    encoding: "utf8"
  });
  execFileSync("git", ["config", "user.name", "Workflow Tester"], {
    cwd: tempRoot,
    encoding: "utf8"
  });
  execFileSync("git", ["config", "user.email", "workflow@test.local"], {
    cwd: tempRoot,
    encoding: "utf8"
  });
  execFileSync("git", ["config", "core.autocrlf", "false"], {
    cwd: tempRoot,
    encoding: "utf8"
  });

  writeFileSync(path.join(tempRoot, "README.md"), "# temp repo\n", "utf8");
  execFileSync("git", ["add", "README.md"], { cwd: tempRoot, encoding: "utf8" });
  execFileSync("git", ["commit", "-m", "chore: init repo"], {
    cwd: tempRoot,
    encoding: "utf8"
  });

  return tempRoot;
}

function runCli(args, cwd = repoRoot) {
  return execFileSync(process.execPath, [worktreeCliPath, ...args], {
    cwd,
    encoding: "utf8"
  });
}

test("create creates a branch worktree under the repo-local .worktrees directory", () => {
  const tempRepo = createTempGitRepo();

  const stdout = runCli([
    "create",
    "--repo-root",
    tempRepo,
    "--branch",
    "feature/cache-index",
    "--base",
    "main"
  ]);
  const result = JSON.parse(stdout);

  assert.equal(result.action, "create");
  assert.equal(result.branch, "feature/cache-index");
  assert.equal(
    result.worktreePath,
    path.join(tempRepo, ".worktrees", "feature-cache-index")
  );
  assert.equal(existsSync(result.worktreePath), true);
  assert.equal(
    execFileSync("git", ["rev-parse", "--abbrev-ref", "HEAD"], {
      cwd: result.worktreePath,
      encoding: "utf8"
    }).trim(),
    "feature/cache-index"
  );
});

test("merge merges the worktree branch back into the target branch", () => {
  const tempRepo = createTempGitRepo();
  const created = JSON.parse(
    runCli([
      "create",
      "--repo-root",
      tempRepo,
      "--branch",
      "feature/session-control",
      "--base",
      "main"
    ])
  );

  writeFileSync(
    path.join(created.worktreePath, "README.md"),
    "# temp repo\n\nsession control merged\n",
    "utf8"
  );
  execFileSync("git", ["add", "README.md"], {
    cwd: created.worktreePath,
    encoding: "utf8"
  });
  execFileSync("git", ["commit", "-m", "feat: add session control"], {
    cwd: created.worktreePath,
    encoding: "utf8"
  });

  const stdout = runCli([
    "merge",
    "--repo-root",
    tempRepo,
    "--branch",
    "feature/session-control",
    "--into",
    "main"
  ]);
  const result = JSON.parse(stdout);

  assert.equal(result.action, "merge");
  assert.equal(result.branch, "feature/session-control");
  assert.equal(result.into, "main");
  assert.match(
    readFileSync(path.join(tempRepo, "README.md"), "utf8"),
    /session control merged/
  );
});

test("delete removes the worktree directory and unregisters it from git", () => {
  const tempRepo = createTempGitRepo();
  const created = JSON.parse(
    runCli([
      "create",
      "--repo-root",
      tempRepo,
      "--branch",
      "feature/worktree-delete",
      "--base",
      "main"
    ])
  );

  const stdout = runCli([
    "delete",
    "--repo-root",
    tempRepo,
    "--branch",
    "feature/worktree-delete"
  ]);
  const result = JSON.parse(stdout);
  const listed = execFileSync("git", ["worktree", "list", "--porcelain"], {
    cwd: tempRepo,
    encoding: "utf8"
  });

  assert.equal(result.action, "delete");
  assert.equal(existsSync(created.worktreePath), false);
  assert.ok(!listed.includes(created.worktreePath));
});

test("recycle reuses an existing clean worktree for the same branch", () => {
  const tempRepo = createTempGitRepo();
  const created = JSON.parse(
    runCli([
      "create",
      "--repo-root",
      tempRepo,
      "--branch",
      "feature/recycle-me",
      "--base",
      "main"
    ])
  );

  const stdout = runCli([
    "recycle",
    "--repo-root",
    tempRepo,
    "--branch",
    "feature/recycle-me",
    "--base",
    "main"
  ]);
  const result = JSON.parse(stdout);

  assert.equal(result.action, "recycle");
  assert.equal(result.reused, true);
  assert.equal(result.worktreePath, created.worktreePath);
});

test("recycle prunes a stale worktree entry and recreates the directory", () => {
  const tempRepo = createTempGitRepo();
  const created = JSON.parse(
    runCli([
      "create",
      "--repo-root",
      tempRepo,
      "--branch",
      "feature/recycle-stale",
      "--base",
      "main"
    ])
  );

  rmSync(created.worktreePath, { recursive: true, force: true });

  const stdout = runCli([
    "recycle",
    "--repo-root",
    tempRepo,
    "--branch",
    "feature/recycle-stale",
    "--base",
    "main"
  ]);
  const result = JSON.parse(stdout);

  assert.equal(result.action, "recycle");
  assert.equal(result.reused, false);
  assert.equal(existsSync(result.worktreePath), true);
});
