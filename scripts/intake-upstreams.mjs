import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildArtifactPlan,
  loadUpstreamCatalog,
  writeArtifactPlan
} from "./lib/upstream-intake.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const defaultRepoRoot = path.resolve(scriptDirectory, "..");
const defaultCatalog = path.join(
  defaultRepoRoot,
  "third_party",
  "upstreams",
  "catalog.json"
);

function parseArgs(argv) {
  const options = {
    catalog: defaultCatalog,
    repoRoot: defaultRepoRoot,
    dryRun: false
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];

    if (argument === "--dry-run") {
      options.dryRun = true;
      continue;
    }

    if (argument === "--catalog") {
      options.catalog = path.resolve(argv[index + 1]);
      index += 1;
      continue;
    }

    if (argument === "--repo-root") {
      options.repoRoot = path.resolve(argv[index + 1]);
      index += 1;
      continue;
    }

    throw new Error(`Unsupported argument: ${argument}`);
  }

  return options;
}

function printSummary(artifacts, dryRun) {
  console.log(dryRun ? "DRY RUN" : "INTAKE");

  for (const artifact of artifacts) {
    console.log(`${dryRun ? "PLAN" : "WROTE"} ${artifact.path}`);
  }
}

try {
  const options = parseArgs(process.argv.slice(2));
  const catalog = await loadUpstreamCatalog(options.catalog);
  const artifacts = buildArtifactPlan({
    entries: catalog.entries,
    repoRoot: options.repoRoot,
    catalogLastReviewedAt: catalog.lastReviewedAt
  });

  if (!options.dryRun) {
    await writeArtifactPlan(artifacts);
  }

  printSummary(artifacts, options.dryRun);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
