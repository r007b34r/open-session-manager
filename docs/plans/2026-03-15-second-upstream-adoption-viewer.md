# Second Upstream Adoption Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver OSM's second real upstream-inspired adoption by upgrading the session detail workspace with viewer-style transcript highlights and todo evidence, drawing on the interaction model documented for `d-kimuson/claude-code-viewer`.

**Architecture:** Promote transcript digest data to a shared Rust capability instead of keeping it trapped inside Markdown export. Expose that digest through the dashboard snapshot, then render it in the React session detail panel with bilingual labels and responsive layout so the feature is visible both in the app and exported artifacts.

**Tech Stack:** Rust, React, TypeScript, Vitest, Playwright, Markdown

---

### Task 1: Add A Shared Transcript Digest Contract

**Files:**
- Create: `src-tauri/src/transcript/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest -- --exact`

**Step 1: Write the failing test**

Add a transcript-digest-focused test that expects todos and highlights to be derived from session sources without relying on export-only helpers.

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest -- --exact`
Expected: FAIL after moving the expectation to a shared module boundary because the digest capability does not exist yet.

**Step 3: Write minimal implementation**

Introduce shared digest types and extraction functions for Codex, Claude Code, and OpenCode.

**Step 4: Run test to verify it passes**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/transcript/mod.rs src-tauri/src/lib.rs src-tauri/src/actions/tests.rs
git commit -m "feat: add shared transcript digest capability"
```

### Task 2: Surface Digest Data In The Dashboard Snapshot

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe test commands::dashboard::tests::fixture_snapshot_includes_transcript_digest -- --exact`

**Step 1: Write the failing test**

Add a dashboard snapshot test that expects session detail records to include transcript highlights and todo items.

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test commands::dashboard::tests::fixture_snapshot_includes_transcript_digest -- --exact`
Expected: FAIL because the snapshot schema does not expose transcript detail yet.

**Step 3: Write minimal implementation**

Attach the shared transcript digest to the indexed session detail records and keep the existing summary and artifact signals intact.

**Step 4: Run test to verify it passes**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test commands::dashboard::tests::fixture_snapshot_includes_transcript_digest -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs
git commit -m "feat: surface transcript digest in dashboard snapshot"
```

### Task 3: Render Viewer-Style Detail Panels In React

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/app.test.tsx`
- Modify: `web/src/styles.css`
- Test: `npm --prefix web run test -- src/app.test.tsx`

**Step 1: Write the failing test**

Add a UI test that expects the selected session detail view to show transcript highlights and todo snapshot content.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- src/app.test.tsx`
Expected: FAIL because the UI does not yet render digest panels.

**Step 3: Write minimal implementation**

Extend the API types, fixtures, i18n copy, and detail component so the new digest data appears as responsive detail cards.

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- src/app.test.tsx`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/i18n.tsx web/src/components/session-detail.tsx web/src/app.test.tsx web/src/styles.css
git commit -m "feat: add viewer-style transcript detail panels"
```

### Task 4: Record The Second Adoption Trail

**Files:**
- Modify: `third_party/upstreams/catalog.json`
- Modify: `tests/upstream-intake/fixtures/catalog.json`
- Modify: `tests/upstream-intake/upstream-intake.test.mjs`
- Modify: `README.md`
- Modify: `docs/release/github-release-notes.md`
- Test: `node --test tests/upstream-intake/upstream-intake.test.mjs`
- Test: `node scripts/intake-upstreams.mjs`

**Step 1: Write the failing test**

Extend upstream intake verification to expect the `d-kimuson/claude-code-viewer` entry to record the adopted transcript detail capability.

**Step 2: Run test to verify it fails**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: FAIL because the adoption metadata is missing.

**Step 3: Write minimal implementation**

Update the catalog and regenerate the upstream docs and attribution files.

**Step 4: Run test to verify it passes**

Run: `node --test tests/upstream-intake/upstream-intake.test.mjs`
Expected: PASS.

**Step 5: Commit**

```bash
git add third_party/upstreams/catalog.json tests/upstream-intake/fixtures/catalog.json tests/upstream-intake/upstream-intake.test.mjs README.md docs/release/github-release-notes.md docs/research/upstreams docs/release/open-source-attribution.md
git commit -m "docs: record second upstream adoption"
```

### Task 5: Verify The Whole Repository

**Files:**
- Test: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe clippy --all-targets --all-features -- -D warnings`

**Step 1: Write the failing test**

Run the full repository verification after the new snapshot and UI fields are added.

**Step 2: Run test to verify it fails**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: FAIL until all new fields and tests are wired.

**Step 3: Write minimal implementation**

Fix any remaining issues and regenerate derived artifacts.

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS.

**Step 5: Commit**

```bash
git add .
git commit -m "feat: adopt viewer-style transcript detail panels"
```
