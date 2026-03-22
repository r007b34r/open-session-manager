import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";

function ensureNonEmptyString(value, fieldName) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`Expected ${fieldName} to be a non-empty string.`);
  }

  return value.trim();
}

function parseArgs(argv) {
  const [action, ...rest] = argv;
  const options = {
    action: ensureNonEmptyString(action, "action")
  };

  for (let index = 0; index < rest.length; index += 1) {
    const argument = rest[index];

    if (!argument.startsWith("--")) {
      throw new Error(`Unsupported argument: ${argument}`);
    }

    const key = argument.slice(2);
    const value = rest[index + 1];

    if (value === undefined || value.startsWith("--")) {
      throw new Error(`Missing value for argument: ${argument}`);
    }

    options[key] = value;
    index += 1;
  }

  return normalizeOptions(options);
}

function normalizeOptions(options) {
  const action = ensureNonEmptyString(options.action, "action");
  const repoRoot = path.resolve(
    ensureNonEmptyString(options["repo-root"] ?? options.repoRoot ?? process.cwd(), "repoRoot")
  );
  const branch = ensureNonEmptyString(options.branch, "branch");
  const worktreeRoot = path.join(repoRoot, ".worktrees");
  const worktreePath = path.join(worktreeRoot, slugifyBranch(branch));

  return {
    action,
    repoRoot,
    branch,
    base: options.base ? ensureNonEmptyString(options.base, "base") : undefined,
    into: options.into ? ensureNonEmptyString(options.into, "into") : undefined,
    worktreeRoot,
    worktreePath
  };
}

function slugifyBranch(branch) {
  return branch
    .replace(/[^A-Za-z0-9._-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");
}

function runGit(repoRoot, args, options = {}) {
  return execFileSync("git", args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: options.capture === false ? "inherit" : ["ignore", "pipe", "pipe"]
  }).trimEnd();
}

function runGitStatus(repoRoot, args) {
  try {
    execFileSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"]
    });
    return true;
  } catch {
    return false;
  }
}

function listWorktrees(repoRoot) {
  const output = runGit(repoRoot, ["worktree", "list", "--porcelain"]);
  const entries = [];
  let current = null;

  for (const rawLine of output.split(/\r?\n/)) {
    const line = rawLine.trim();

    if (line === "") {
      if (current) {
        entries.push(current);
        current = null;
      }
      continue;
    }

    const [key, ...rest] = line.split(" ");
    const value = rest.join(" ");

    if (key === "worktree") {
      current = { worktreePath: path.resolve(value) };
      continue;
    }

    if (!current) {
      continue;
    }

    if (key === "branch") {
      current.branchRef = value;
      continue;
    }

    if (key === "HEAD") {
      current.head = value;
      continue;
    }

    current[key] = value || true;
  }

  if (current) {
    entries.push(current);
  }

  return entries;
}

function branchRef(branch) {
  return `refs/heads/${branch}`;
}

function findWorktreeForBranch(repoRoot, branch) {
  const expectedRef = branchRef(branch);

  return listWorktrees(repoRoot).find((entry) => entry.branchRef === expectedRef);
}

function ensureRepoRoot(repoRoot) {
  if (!runGitStatus(repoRoot, ["rev-parse", "--show-toplevel"])) {
    throw new Error(`Not a git repository: ${repoRoot}`);
  }
}

function isBranchDefined(repoRoot, branch) {
  return runGitStatus(repoRoot, ["show-ref", "--verify", "--quiet", branchRef(branch)]);
}

function isWorktreeClean(worktreePath) {
  return runGit(worktreePath, ["status", "--porcelain"]).trim() === "";
}

function pruneWorktrees(repoRoot) {
  runGit(repoRoot, ["worktree", "prune"]);
}

function createWorktree(options) {
  const existing = findWorktreeForBranch(options.repoRoot, options.branch);

  if (existing?.worktreePath && !existsSync(existing.worktreePath)) {
    pruneWorktrees(options.repoRoot);
  }

  if (existsSync(options.worktreePath)) {
    throw new Error(`Worktree path already exists: ${options.worktreePath}`);
  }

  mkdirSync(options.worktreeRoot, { recursive: true });

  if (isBranchDefined(options.repoRoot, options.branch)) {
    runGit(options.repoRoot, ["worktree", "add", options.worktreePath, options.branch]);
  } else {
    const base = ensureNonEmptyString(options.base, "base");
    runGit(options.repoRoot, [
      "worktree",
      "add",
      options.worktreePath,
      "-b",
      options.branch,
      base
    ]);
  }

  return {
    action: "create",
    branch: options.branch,
    base: options.base ?? null,
    worktreePath: options.worktreePath
  };
}

function mergeWorktree(options) {
  const into = ensureNonEmptyString(options.into, "into");

  runGit(options.repoRoot, ["checkout", into]);
  runGit(options.repoRoot, ["merge", "--no-ff", "--no-edit", options.branch]);

  return {
    action: "merge",
    branch: options.branch,
    into,
    worktreePath: findWorktreeForBranch(options.repoRoot, options.branch)?.worktreePath ?? null
  };
}

function deleteWorktree(options) {
  const registered = findWorktreeForBranch(options.repoRoot, options.branch);
  const worktreePath = registered?.worktreePath ?? options.worktreePath;

  if (registered?.worktreePath && existsSync(registered.worktreePath)) {
    runGit(options.repoRoot, ["worktree", "remove", registered.worktreePath, "--force"]);
  } else {
    pruneWorktrees(options.repoRoot);
    if (existsSync(worktreePath)) {
      rmSync(worktreePath, { recursive: true, force: true });
    }
  }

  return {
    action: "delete",
    branch: options.branch,
    worktreePath,
    removed: !existsSync(worktreePath)
  };
}

function recycleWorktree(options) {
  const registered = findWorktreeForBranch(options.repoRoot, options.branch);

  if (registered?.worktreePath && existsSync(registered.worktreePath)) {
    if (!isWorktreeClean(registered.worktreePath)) {
      throw new Error(`Cannot recycle a dirty worktree: ${registered.worktreePath}`);
    }

    return {
      action: "recycle",
      branch: options.branch,
      base: options.base ?? null,
      worktreePath: registered.worktreePath,
      reused: true
    };
  }

  if (registered?.worktreePath && !existsSync(registered.worktreePath)) {
    pruneWorktrees(options.repoRoot);
  }

  const created = createWorktree(options);

  return {
    action: "recycle",
    branch: created.branch,
    base: created.base,
    worktreePath: created.worktreePath,
    reused: false
  };
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  ensureRepoRoot(options.repoRoot);

  let result;

  switch (options.action) {
    case "create":
      result = createWorktree(options);
      break;
    case "merge":
      result = mergeWorktree(options);
      break;
    case "delete":
      result = deleteWorktree(options);
      break;
    case "recycle":
      result = recycleWorktree(options);
      break;
    default:
      throw new Error(`Unsupported action: ${options.action}`);
  }

  process.stdout.write(`${JSON.stringify(result)}\n`);
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
