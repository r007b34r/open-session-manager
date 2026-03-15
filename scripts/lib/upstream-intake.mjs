import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const VALID_ABSORPTION_MODES = new Set([
  "candidate-absorb",
  "reference-only"
]);

function ensureString(value, fieldName) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`Expected ${fieldName} to be a non-empty string.`);
  }

  return value.trim();
}

function ensureStringArray(value, fieldName) {
  if (!Array.isArray(value)) {
    throw new Error(`Expected ${fieldName} to be an array.`);
  }

  return value.map((item, index) =>
    ensureString(item, `${fieldName}[${index}]`)
  );
}

function slugifyRepo(repo) {
  return repo.toLowerCase().replace(/[\\/]/g, "-");
}

function normalizeEvidence(entry) {
  if (!Array.isArray(entry.evidence) || entry.evidence.length === 0) {
    throw new Error(`Expected ${entry.repo} to include at least one evidence link.`);
  }

  return entry.evidence.map((item, index) => ({
    label: ensureString(item.label, `${entry.repo}.evidence[${index}].label`),
    url: ensureString(item.url, `${entry.repo}.evidence[${index}].url`)
  }));
}

function normalizeEntry(entry, index) {
  const repo = ensureString(entry.repo, `entries[${index}].repo`);
  const name = ensureString(entry.name ?? repo.split("/").at(-1), `${repo}.name`);
  const absorptionMode = ensureString(
    entry.absorption?.mode,
    `${repo}.absorption.mode`
  );

  if (!VALID_ABSORPTION_MODES.has(absorptionMode)) {
    throw new Error(`Unsupported absorption mode for ${repo}: ${absorptionMode}`);
  }

  return {
    repo,
    slug: ensureString(entry.slug ?? slugifyRepo(repo), `${repo}.slug`),
    name,
    homepage: ensureString(entry.homepage, `${repo}.homepage`),
    license: {
      spdx: ensureString(entry.license?.spdx, `${repo}.license.spdx`),
      notes: ensureString(entry.license?.notes, `${repo}.license.notes`)
    },
    absorption: {
      mode: absorptionMode,
      rationale: ensureString(
        entry.absorption?.rationale,
        `${repo}.absorption.rationale`
      ),
      blockedBy: ensureStringArray(
        entry.absorption?.blockedBy ?? [],
        `${repo}.absorption.blockedBy`
      )
    },
    reviewStatus: ensureString(entry.reviewStatus, `${repo}.reviewStatus`),
    reviewedAt: ensureString(entry.reviewedAt, `${repo}.reviewedAt`),
    summary: ensureString(entry.summary, `${repo}.summary`),
    projectShape: ensureString(entry.projectShape, `${repo}.projectShape`),
    stackSignals: ensureStringArray(entry.stackSignals ?? [], `${repo}.stackSignals`),
    focusAreas: ensureStringArray(entry.focusAreas ?? [], `${repo}.focusAreas`),
    verifiedPaths: ensureStringArray(
      entry.verifiedPaths ?? [],
      `${repo}.verifiedPaths`
    ),
    inspectionTargets: ensureStringArray(
      entry.inspectionTargets ?? [],
      `${repo}.inspectionTargets`
    ),
    integrationTargets: ensureStringArray(
      entry.integrationTargets ?? [],
      `${repo}.integrationTargets`
    ),
    releaseAcknowledgement: ensureString(
      entry.releaseAcknowledgement,
      `${repo}.releaseAcknowledgement`
    ),
    constraints: ensureStringArray(entry.constraints ?? [], `${repo}.constraints`),
    adoptedCapabilities: ensureStringArray(
      entry.adoptedCapabilities ?? [],
      `${repo}.adoptedCapabilities`
    ),
    upstreamSourceFiles: ensureStringArray(
      entry.upstreamSourceFiles ?? [],
      `${repo}.upstreamSourceFiles`
    ),
    evidence: normalizeEvidence({
      repo,
      evidence: entry.evidence
    })
  };
}

function formatList(items) {
  if (items.length === 0) {
    return "- None";
  }

  return items.map((item) => `- ${item}`).join("\n");
}

function buildEvidenceList(entry) {
  return entry.evidence.map((item) => `- [${item.label}](${item.url})`).join("\n");
}

export async function loadUpstreamCatalog(catalogPath) {
  const raw = JSON.parse(await readFile(catalogPath, "utf8"));

  if (!Number.isInteger(raw.version)) {
    throw new Error("Expected catalog version to be an integer.");
  }

  if (!Array.isArray(raw.entries) || raw.entries.length === 0) {
    throw new Error("Expected catalog entries to be a non-empty array.");
  }

  return {
    version: raw.version,
    lastReviewedAt: ensureString(raw.lastReviewedAt, "lastReviewedAt"),
    entries: raw.entries
      .map((entry, index) => normalizeEntry(entry, index))
      .sort((left, right) => left.repo.localeCompare(right.repo))
  };
}

export function resolveMirrorDirectory(entry, repoRoot) {
  return path.join(repoRoot, "third_party", "upstreams", "mirrors", entry.slug);
}

export function buildUpstreamResearchReport(entry) {
  return `# ${entry.repo}

- Canonical URL: ${entry.homepage}
- License: ${entry.license.spdx}
- Review Status: ${entry.reviewStatus}
- Reviewed At: ${entry.reviewedAt}
- Absorption Mode: ${entry.absorption.mode}

## Summary

${entry.summary}

## Why It Matters To OSM

${entry.absorption.rationale}

## Project Shape

- ${entry.projectShape}
- Stack Signals:
${formatList(entry.stackSignals)}

## Verified Paths

${formatList(entry.verifiedPaths)}

## Inspection Targets

${formatList(entry.inspectionTargets)}

## Integration Targets

${formatList(entry.integrationTargets)}

## Adopted Capabilities

${formatList(entry.adoptedCapabilities)}

## Upstream Source Files

${formatList(entry.upstreamSourceFiles)}

## Constraints

${formatList(entry.constraints.concat(entry.absorption.blockedBy))}

## Release Acknowledgement

${entry.releaseAcknowledgement}

## Evidence

${buildEvidenceList(entry)}
`;
}

export function buildUpstreamResearchIndex(entries) {
  const rows = entries
    .map(
      (entry) =>
        `| ${entry.repo} | ${entry.license.spdx} | ${entry.absorption.mode} | ${entry.reviewStatus} | ${entry.summary} |`
    )
    .join("\n");

  return `# OSM Upstream Research Index

This index is generated from \`third_party/upstreams/catalog.json\`.

| Repository | License | Absorption | Review | Summary |
| --- | --- | --- | --- | --- |
${rows}
`;
}

export function buildOpenSourceAttribution(entries) {
  const absorbCandidates = entries.filter(
    (entry) => entry.absorption.mode === "candidate-absorb"
  );
  const referenceOnly = entries.filter(
    (entry) => entry.absorption.mode === "reference-only"
  );

  const formatAttributionSection = (sectionEntries) =>
    sectionEntries
      .map((entry) => {
        const lines = [
          `- ${entry.repo} (${entry.license.spdx})`,
          `  Posture: ${entry.absorption.mode}`,
          `  Why: ${entry.releaseAcknowledgement}`
        ];

        if (entry.adoptedCapabilities.length > 0) {
          lines.push(`  Adopted: ${entry.adoptedCapabilities.join("; ")}`);
        }

        if (entry.constraints.length > 0 || entry.absorption.blockedBy.length > 0) {
          lines.push(
            `  Constraints: ${entry.constraints
              .concat(entry.absorption.blockedBy)
              .join("; ")}`
          );
        }

        return lines.join("\n");
      })
      .join("\n");

  return `# open Session Manager Open Source Attribution

This file is generated from \`third_party/upstreams/catalog.json\`.

## Candidate Absorb

${formatAttributionSection(absorbCandidates)}

## Reference Only

${formatAttributionSection(referenceOnly)}
`;
}

export function buildIntakeManifest(entries, repoRoot, catalogLastReviewedAt) {
  return JSON.stringify(
    {
      catalogLastReviewedAt,
      entries: entries.map((entry) => ({
        repo: entry.repo,
        slug: entry.slug,
        absorptionMode: entry.absorption.mode,
        license: entry.license.spdx,
        mirrorDirectory: path
          .relative(repoRoot, resolveMirrorDirectory(entry, repoRoot))
          .split(path.sep)
          .join("/")
      }))
    },
    null,
    2
  );
}

export function buildArtifactPlan({ entries, repoRoot, catalogLastReviewedAt }) {
  const researchRoot = path.join(repoRoot, "docs", "research", "upstreams");
  const artifacts = entries.map((entry) => ({
    path: path.join(researchRoot, `${entry.slug}.md`),
    content: buildUpstreamResearchReport(entry)
  }));

  artifacts.push({
    path: path.join(researchRoot, "index.md"),
    content: buildUpstreamResearchIndex(entries)
  });
  artifacts.push({
    path: path.join(repoRoot, "docs", "release", "open-source-attribution.md"),
    content: buildOpenSourceAttribution(entries)
  });
  artifacts.push({
    path: path.join(repoRoot, "third_party", "upstreams", "intake-manifest.json"),
    content: buildIntakeManifest(entries, repoRoot, catalogLastReviewedAt)
  });

  return artifacts;
}

export async function writeArtifactPlan(artifacts) {
  for (const artifact of artifacts) {
    await mkdir(path.dirname(artifact.path), { recursive: true });
    await writeFile(artifact.path, artifact.content, "utf8");
  }
}
