import { access, mkdir, readFile, writeFile } from "node:fs/promises";
import { constants as fsConstants } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

import {
  diffFixtureSnapshots,
  normalizeFixtureSnapshot
} from "./lib/fixture-snapshot.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const args = process.argv.slice(2);
const fixturesRoot = path.resolve(repoRoot, getFlagValue(args, "--fixtures") ?? "tests/fixtures");
const goldenPath = path.resolve(
  repoRoot,
  getFlagValue(args, "--golden") ?? "tests/fixtures/dashboard-snapshot.golden.json"
);

const actualSnapshot = normalizeFixtureSnapshot(
  JSON.parse(await runFixtureSnapshot(fixturesRoot))
);

if (args.includes("--write")) {
  await mkdir(path.dirname(goldenPath), { recursive: true });
  await writeFile(goldenPath, `${JSON.stringify(actualSnapshot, null, 2)}\n`);
  console.log(`WROTE ${goldenPath}`);
  process.exit(0);
}

const expectedSnapshot = normalizeFixtureSnapshot(
  JSON.parse(await readFile(goldenPath, "utf8"))
);
const differences = diffFixtureSnapshots(expectedSnapshot, actualSnapshot);

if (differences.length === 0) {
  console.log(
    JSON.stringify(
      {
        status: "ok",
        goldenPath,
        fixturesRoot
      },
      null,
      2
    )
  );
  process.exit(0);
}

console.error(
  JSON.stringify(
    {
      status: "drift",
      goldenPath,
      fixturesRoot,
      differenceCount: differences.length,
      differences: differences.slice(0, 50)
    },
    null,
    2
  )
);
process.exit(1);

async function runFixtureSnapshot(fixturesPath) {
  const cargo = await resolveCargoCommand();
  return runCommand(
    cargo,
    [
      "run",
      "-p",
      "open-session-manager-core",
      "--",
      "snapshot",
      "--fixtures",
      fixturesPath
    ],
    repoRoot
  );
}

async function resolveCargoCommand() {
  const candidates = [
    process.env.CARGO,
    path.join(homedir(), ".cargo", "bin", process.platform === "win32" ? "cargo.exe" : "cargo"),
    path.join(
      homedir(),
      ".rustup",
      "toolchains",
      process.platform === "win32"
        ? "stable-x86_64-pc-windows-msvc"
        : "stable-x86_64-unknown-linux-gnu",
      "bin",
      process.platform === "win32" ? "cargo.exe" : "cargo"
    ),
    process.platform === "win32" ? "cargo.exe" : "cargo"
  ].filter(Boolean);

  for (const candidate of candidates) {
    if (await isExecutable(candidate)) {
      return candidate;
    }
  }

  throw new Error("cargo executable was not found in PATH or common local Rust locations.");
}

async function isExecutable(candidate) {
  if (!candidate) {
    return false;
  }

  if (!candidate.includes(path.sep)) {
    return true;
  }

  try {
    await access(candidate, fsConstants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function runCommand(command, commandArgs, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, commandArgs, {
      cwd,
      stdio: ["ignore", "pipe", "pipe"],
      shell: false
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });

    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });

    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(stderr || `${command} exited with code ${code}`));
      }
    });
  });
}

function getFlagValue(argv, flag) {
  const index = argv.indexOf(flag);
  if (index === -1 || index === argv.length - 1) {
    return null;
  }

  return argv[index + 1];
}
