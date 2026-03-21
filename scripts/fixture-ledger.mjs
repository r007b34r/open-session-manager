import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  detectFixtureLedgerDrift,
  generateFixtureLedger,
  readFixtureLedger,
  writeFixtureLedger
} from "./lib/fixture-ledger.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const args = process.argv.slice(2);
const fixturesRoot = path.resolve(repoRoot, getFlagValue(args, "--fixtures") ?? "tests/fixtures");
const ledgerPath = path.resolve(
  repoRoot,
  getFlagValue(args, "--ledger") ?? "tests/fixtures/fixture-ledger.json"
);

if (args.includes("--write")) {
  const ledger = await generateFixtureLedger(fixturesRoot);
  await writeFixtureLedger(ledgerPath, ledger);
  console.log(`WROTE ${ledgerPath}`);
  process.exit(0);
}

const baselineLedger = await readFixtureLedger(ledgerPath);
const drift = await detectFixtureLedgerDrift(baselineLedger, fixturesRoot);

if (drift.status === "ok") {
  console.log(
    JSON.stringify(
      {
        status: "ok",
        ledgerPath,
        fixturesRoot,
        fixtureCount: drift.currentLedger.fixtures.length
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
      ledgerPath,
      fixturesRoot,
      addedFixtures: drift.addedFixtures,
      removedFixtures: drift.removedFixtures,
      changed: drift.changed
    },
    null,
    2
  )
);
process.exit(1);

function getFlagValue(argv, flag) {
  const index = argv.indexOf(flag);
  if (index === -1) {
    return null;
  }

  return argv[index + 1] ?? null;
}
