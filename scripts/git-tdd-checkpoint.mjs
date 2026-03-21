import { normalizeCheckpointOptions, recordCheckpoint } from "./lib/git-workflow.mjs";

function parseArgs(argv) {
  const options = {
    commands: [],
    dryRun: false
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];

    if (argument === "--repo-root") {
      options.repoRoot = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--item") {
      options.itemId = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--phase") {
      options.phase = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--note") {
      options.note = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--command") {
      options.commands.push(argv[index + 1]);
      index += 1;
      continue;
    }

    if (argument === "--output") {
      options.outputPath = argv[index + 1];
      index += 1;
      continue;
    }

    if (argument === "--dry-run") {
      options.dryRun = true;
      continue;
    }

    throw new Error(`Unsupported argument: ${argument}`);
  }

  return normalizeCheckpointOptions(options);
}

try {
  const options = parseArgs(process.argv.slice(2));
  const result = await recordCheckpoint(options);

  if (result.dryRun) {
    console.log("DRY RUN");
  } else {
    console.log("RECORDED CHECKPOINT");
  }

  console.log(`Item: ${result.itemId}`);
  console.log(`Phase: ${result.phase}`);
  console.log(`Tag: ${result.tagName}`);
  console.log(`Notes Ref: ${result.noteRef}`);
  console.log(`Review: ${result.reviewPath}`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
