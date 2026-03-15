import { access } from "node:fs/promises";
import { constants as fsConstants } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "..");
const args = process.argv.slice(2);

if (args.length === 0) {
  throw new Error("usage: node scripts/run-tauri.mjs <dev|build> [args...]");
}

const cargo = await resolveCargoCommand();
const cargoDir = cargo.includes(path.sep) ? path.dirname(cargo) : null;
const tauriArgs = withDefaultConfig(args);
const env = { ...process.env };

if (cargoDir) {
  env.PATH = `${cargoDir}${path.delimiter}${env.PATH ?? ""}`;
}

await runCommand(
  "npx",
  ["--prefix", "web", "tauri", ...tauriArgs],
  repoRoot,
  env
);

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

function withDefaultConfig(argv) {
  if (argv.includes("--config") || argv.includes("-c")) {
    return argv;
  }

  return [...argv, "--config", "src-tauri/tauri.conf.json"];
}

function runCommand(command, commandArgs, cwd, env) {
  return new Promise((resolve, reject) => {
    const invocation = buildInvocation(command, commandArgs);
    const child = spawn(invocation.command, invocation.args, {
      cwd,
      env,
      stdio: "inherit",
      shell: false
    });

    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`${command} exited with code ${code}`));
      }
    });
  });
}

function buildInvocation(command, args) {
  if (process.platform !== "win32") {
    return { command, args };
  }

  const cmdLine = [command, ...args].map(quoteWindowsArg).join(" ");
  return {
    command: process.env.ComSpec ?? "cmd.exe",
    args: ["/d", "/s", "/c", cmdLine]
  };
}

function quoteWindowsArg(value) {
  if (/^[A-Za-z0-9_./:=\\-]+$/.test(value)) {
    return value;
  }

  return `"${value.replace(/(\\*)"/g, '$1$1\\"').replace(/(\\+)$/g, "$1$1")}"`;
}
