# Upstream Intake Pipeline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn competitor research into a repeatable upstream intake pipeline with a structured catalog, attribution-safe license handling, generated research artifacts, and verification coverage.

**Architecture:** Keep upstream governance outside the product runtime and implement it as a repository-level intake toolchain. Store curated upstream metadata in a structured catalog, generate human-readable research and release attribution from that catalog, and support optional local mirrors for deeper code review without forcing third-party code into the shipping app.

**Tech Stack:** Node.js, native `node:test`, Markdown, JSON, PowerShell

---

### Task 1: Define The Upstream Catalog And Governance Surface

**Files:**
- Create: `third_party/upstreams/catalog.json`
- Create: `docs/research/upstreams/README.md`
- Modify: `README.md`

**Step 1: Write the failing test**

Add a catalog validation test that expects every upstream entry to include:

- repository slug and canonical URL
- license policy and absorption posture
- candidate code paths to inspect
- attribution and release-note linkage

**Step 2: Run test to verify it fails**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: FAIL because the repository does not yet define a structured upstream catalog.

**Step 3: Write minimal implementation**

Create the initial catalog for the shortlisted competitors and document:

- which repos are safe to absorb directly
- which repos must stay reference-only
- why each repo matters to OSM

**Step 4: Run test to verify it passes**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: PASS with the catalog loading successfully.

**Step 5: Commit**

```bash
git add third_party/upstreams/catalog.json docs/research/upstreams/README.md README.md
git commit -m "docs: add upstream governance catalog"
```

### Task 2: Add A Deterministic Intake Library With Dry-Run Support

**Files:**
- Create: `scripts/lib/upstream-intake.mjs`
- Create: `tests/upstream-intake/upstream-intake.test.mjs`
- Test: `node --test tests/upstream-intake/upstream-intake.test.mjs`

**Step 1: Write the failing test**

Add tests that expect the intake library to:

- normalize upstream entries into local slugs and output paths
- generate release attribution rows from the catalog
- emit deterministic markdown research summaries without network access

**Step 2: Run test to verify it fails**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: FAIL because the intake library does not exist yet.

**Step 3: Write minimal implementation**

Implement pure helpers for:

- loading and validating catalog JSON
- building per-repo research summaries
- building release acknowledgement content
- resolving local mirror directories

**Step 4: Run test to verify it passes**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: PASS with deterministic output.

**Step 5: Commit**

```bash
git add scripts/lib/upstream-intake.mjs tests/upstream-intake/upstream-intake.test.mjs
git commit -m "feat: add upstream intake library"
```

### Task 3: Add The Repository Intake Command

**Files:**
- Create: `scripts/intake-upstreams.mjs`
- Modify: `scripts/verify.ps1`
- Test: `node scripts/intake-upstreams.mjs --dry-run`

**Step 1: Write the failing test**

Extend the intake tests to expect a CLI entrypoint that can:

- read the catalog
- generate research output into `docs/research/upstreams/`
- generate release attribution into `docs/release/open-source-attribution.md`
- support `--dry-run` and `--catalog`

**Step 2: Run test to verify it fails**

Run: `node scripts/intake-upstreams.mjs --dry-run`
Expected: FAIL because the CLI entrypoint does not exist yet.

**Step 3: Write minimal implementation**

Implement the command so it can:

- create deterministic docs from the catalog without network
- optionally prepare mirror directories for later git clone or fetch usage
- print a concise action summary for each upstream

**Step 4: Run test to verify it passes**

Run: `node scripts/intake-upstreams.mjs --dry-run`
Expected: PASS and show the files that would be written.

**Step 5: Commit**

```bash
git add scripts/intake-upstreams.mjs scripts/verify.ps1
git commit -m "feat: add upstream intake command"
```

### Task 4: Generate First-Class Research And Attribution Artifacts

**Files:**
- Create: `docs/research/upstreams/index.md`
- Create: `docs/release/open-source-attribution.md`
- Modify: `docs/research/2026-03-15-agent-session-landscape.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Write the failing test**

Add expectations that the generated artifacts include:

- direct-absorb vs reference-only classification
- license and risk posture
- candidate modules or code paths
- acknowledgement language for the release surface

**Step 2: Run test to verify it fails**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: FAIL because the generated documents are missing.

**Step 3: Write minimal implementation**

Generate the first artifact set and wire existing docs to point at the new authoritative sources.

**Step 4: Run test to verify it passes**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: PASS with generated content aligned to the catalog.

**Step 5: Commit**

```bash
git add docs/research/upstreams/index.md docs/release/open-source-attribution.md docs/research/2026-03-15-agent-session-landscape.md docs/release/github-release-notes.md
git commit -m "docs: generate upstream research and attribution artifacts"
```

### Task 5: Verify The Whole Intake Flow

**Files:**
- Modify: `README.md`
- Modify: `docs/research/2026-03-15-osm-audit-and-roadmap.md`
- Test: `node --test tests/upstream-intake/upstream-intake.test.mjs`
- Test: `node scripts/intake-upstreams.mjs`
- Test: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 1: Write the failing test**

Run the end-to-end intake flow and confirm the repository still lacks a documented, verifiable upstream absorption process.

**Step 2: Run test to verify it fails**

Run: `node scripts/intake-upstreams.mjs`
Expected: FAIL or produce incomplete output because the intake flow is not fully wired.

**Step 3: Write minimal implementation**

Finalize docs so contributors can:

- refresh the upstream catalog
- regenerate research and attribution output
- understand the legal and engineering review posture for each competitor

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS with upstream intake verification included.

**Step 5: Commit**

```bash
git add README.md docs/research/2026-03-15-osm-audit-and-roadmap.md scripts/verify.ps1
git commit -m "docs: document upstream intake workflow"
```
