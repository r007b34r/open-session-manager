import { mkdir, writeFile } from "node:fs/promises";
import { access } from "node:fs/promises";
import { constants as fsConstants } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";

const repoRoot = process.cwd();
const args = process.argv.slice(2);
const fixtures = getFlagValue(args, "--fixtures");
const outputPath = path.resolve(
  repoRoot,
  getFlagValue(args, "--output") ?? "web/public/dashboard-snapshot.json"
);

const cargo = await resolveCargoCommand();
const cargoArgs = ["run", "-p", "open-session-manager-core", "--", "snapshot"];

if (fixtures) {
  cargoArgs.push("--fixtures", path.resolve(repoRoot, fixtures));
}

const snapshotJson = await runCommand(cargo, cargoArgs, repoRoot);
await mkdir(path.dirname(outputPath), { recursive: true });
await writeFile(outputPath, snapshotJson, "utf8");

console.log(`snapshot exported to ${outputPath}`);

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
