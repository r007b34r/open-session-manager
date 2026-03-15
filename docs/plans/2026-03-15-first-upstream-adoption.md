# First Upstream Adoption Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver OSM's first real upstream capability adoption by upgrading Markdown export with transcript highlights and cleanup-oriented sections inspired by `daaain/claude-code-log`.

**Architecture:** Keep the adoption clean-room and adapter-aware. Reuse OSM's existing session and insight models, add a transcript digest layer inside the export flow, and record the adoption in the upstream governance catalog so the feature is traceable from implementation back to source inspiration.

**Tech Stack:** Rust, serde_json, Markdown, Node.js catalog tooling

---

### Task 1: Lock The Export Upgrade With Failing Tests

**Files:**
- Modify: `src-tauri/src/actions/tests.rs`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest -- --exact`

**Step 1: Write the failing test**

Add an export test that expects the Markdown artifact to include:

- cleanup recommendation
- tags and risk flags
- transcript highlights section
- Claude todos snapshot when present

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest -- --exact`
Expected: FAIL because the current exporter only emits summary, progress, and source metadata.

**Step 3: Write minimal implementation**

Implement transcript-aware export helpers for Codex, Claude Code, and OpenCode, and render a richer Markdown layout.

**Step 4: Run test to verify it passes**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/actions/tests.rs src-tauri/src/actions/export.rs
git commit -m "feat: adopt richer markdown export digest"
```

### Task 2: Record The Adoption In The Upstream Catalog

**Files:**
- Modify: `third_party/upstreams/catalog.json`
- Modify: `tests/upstream-intake/fixtures/catalog.json`
- Modify: `tests/upstream-intake/upstream-intake.test.mjs`
- Modify: `scripts/lib/upstream-intake.mjs`
- Test: `node --test tests/upstream-intake/upstream-intake.test.mjs`

**Step 1: Write the failing test**

Extend the upstream intake tests to expect a generated report field that records adopted capabilities and upstream source files for `daaain/claude-code-log`.

**Step 2: Run test to verify it fails**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: FAIL because the catalog does not yet record concrete adoption details.

**Step 3: Write minimal implementation**

Add adoption metadata to the catalog and include it in generated research output.

**Step 4: Run test to verify it passes**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: PASS.

**Step 5: Commit**

```bash
git add third_party/upstreams/catalog.json tests/upstream-intake/fixtures/catalog.json tests/upstream-intake/upstream-intake.test.mjs scripts/lib/upstream-intake.mjs
git commit -m "docs: record first upstream adoption"
```

### Task 3: Regenerate Docs And Verify The Whole Repository

**Files:**
- Modify: `docs/release/github-release-notes.md`
- Modify: `README.md`
- Test: `node scripts/intake-upstreams.mjs`
- Test: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe clippy --all-targets --all-features -- -D warnings`

**Step 1: Write the failing test**

Regenerate upstream docs and verify the repository still lacks an explicit statement of the first adopted upstream capability.

**Step 2: Run test to verify it fails**

Run: `node scripts/intake-upstreams.mjs`
Expected: generated docs do not yet mention the first adopted capability.

**Step 3: Write minimal implementation**

Regenerate research and attribution files, then update release-facing docs to point at the adoption.

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS.

**Step 5: Commit**

```bash
git add README.md docs/release/github-release-notes.md docs/research/upstreams docs/release/open-source-attribution.md third_party/upstreams/intake-manifest.json
git commit -m "docs: publish first upstream adoption trail"
```
