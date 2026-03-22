import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..", "..");
const reviewCliPath = path.join(repoRoot, "scripts", "git-review-snapshot.mjs");
const checkpointCliPath = path.join(repoRoot, "scripts", "git-tdd-checkpoint.mjs");

function createTempGitRepo() {
  const tempRoot = mkdtempSync(
    path.join(process.env.TEMP ?? process.cwd(), "osm-git-workflow-")
  );

  execFileSync("git", ["init"], { cwd: tempRoot, encoding: "utf8" });
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

  writeFileSync(path.join(tempRoot, "README.md"), "# temp repo\n\nchanged\n", "utf8");

  return tempRoot;
}

function createTempGitWorktree() {
  const tempRoot = createTempGitRepo();
  const worktreesRoot = path.join(tempRoot, ".worktrees");
  const worktreePath = path.join(worktreesRoot, "feature-a");

  execFileSync("git", ["worktree", "add", "--quiet", worktreePath], {
    cwd: tempRoot,
    encoding: "utf8"
  });

  writeFileSync(
    path.join(worktreePath, "README.md"),
    "# temp repo\n\nfrom worktree\n",
    "utf8"
  );

  return {
    mainRepo: tempRoot,
    worktreePath
  };
}

test("git-review-snapshot CLI writes a stable markdown review file", () => {
  const tempRepo = createTempGitRepo();
  const outputPath = path.join(tempRepo, "review.md");

  const stdout = execFileSync(
    process.execPath,
    [
      reviewCliPath,
      "--repo-root",
      tempRepo,
      "--item",
      "CFG-01",
      "--phase",
      "red",
      "--note",
      "写回表单测试先失败",
      "--command",
      "node --test tests/git-workflow/git-workflow.test.mjs",
      "--output",
      outputPath
    ],
    {
      cwd: repoRoot,
      encoding: "utf8"
    }
  );

  assert.match(stdout, /WROTE REVIEW/);
  assert.equal(existsSync(outputPath), true);

  const content = readFileSync(outputPath, "utf8");
  assert.match(content, /CFG-01/);
  assert.match(content, /Phase: red/);
  assert.match(content, /写回表单测试先失败/);
  assert.match(content, /git status --short --branch/);
});

test("git-tdd-checkpoint CLI dry-run prints tag, note ref, and review path without mutating repo", () => {
  const tempRepo = createTempGitRepo();

  const stdout = execFileSync(
    process.execPath,
    [
      checkpointCliPath,
      "--repo-root",
      tempRepo,
      "--item",
      "TOOL-01",
      "--phase",
      "green",
      "--note",
      "最小实现已转绿",
      "--command",
      "node --test tests/git-workflow/git-workflow.test.mjs",
      "--dry-run"
    ],
    {
      cwd: repoRoot,
      encoding: "utf8"
    }
  );

  assert.match(stdout, /DRY RUN/);
  assert.match(stdout, /refs\/notes\/osm-tdd/);
  assert.match(stdout, /osm\/tdd\/TOOL-01\/green\//);

  const tags = execFileSync("git", ["tag", "--list", "osm/tdd/TOOL-01/green/*"], {
    cwd: tempRepo,
    encoding: "utf8"
  });
  assert.equal(tags.trim(), "");
});

test("git-tdd-checkpoint CLI records annotated tag, git note, and review snapshot", () => {
  const tempRepo = createTempGitRepo();

  const stdout = execFileSync(
    process.execPath,
    [
      checkpointCliPath,
      "--repo-root",
      tempRepo,
      "--item",
      "SRCH-01",
      "--phase",
      "verify",
      "--note",
      "索引验证通过",
      "--command",
      "cargo test --lib -- --test-threads=1"
    ],
    {
      cwd: repoRoot,
      encoding: "utf8"
    }
  );

  assert.match(stdout, /RECORDED CHECKPOINT/);
  assert.match(stdout, /osm\/tdd\/SRCH-01\/verify\//);

  const tags = execFileSync("git", ["tag", "--list", "osm/tdd/SRCH-01/verify/*"], {
    cwd: tempRepo,
    encoding: "utf8"
  });
  assert.notEqual(tags.trim(), "");

  const note = execFileSync(
    "git",
    ["notes", "--ref", "refs/notes/osm-tdd", "show", "HEAD"],
    {
      cwd: tempRepo,
      encoding: "utf8"
    }
  );
  assert.match(note, /SRCH-01/);
  assert.match(note, /verify/);
  assert.match(note, /索引验证通过/);

  const match = stdout.match(/Review: (?<reviewPath>.+)/);
  assert.ok(match?.groups?.reviewPath);
  assert.equal(existsSync(match.groups.reviewPath), true);
});

test("git-tdd-checkpoint CLI also works from a git worktree where .git is a file", () => {
  const { worktreePath } = createTempGitWorktree();

  const stdout = execFileSync(
    process.execPath,
    [
      checkpointCliPath,
      "--repo-root",
      worktreePath,
      "--item",
      "TOOL-01",
      "--phase",
      "review",
      "--note",
      "worktree checkpoint",
      "--command",
      "git status --short --branch"
    ],
    {
      cwd: repoRoot,
      encoding: "utf8"
    }
  );

  assert.match(stdout, /RECORDED CHECKPOINT/);
  const match = stdout.match(/Review: (?<reviewPath>.+)/);
  assert.ok(match?.groups?.reviewPath);
  assert.equal(existsSync(match.groups.reviewPath), true);

  const note = execFileSync(
    "git",
    ["notes", "--ref", "refs/notes/osm-tdd", "show", "HEAD"],
    {
      cwd: worktreePath,
      encoding: "utf8"
    }
  );
  assert.match(note, /worktree checkpoint/);
});
