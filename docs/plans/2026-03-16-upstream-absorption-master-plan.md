# Upstream Absorption Master Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the next upstream absorption tranche into code by adding real usage/cost analytics to OSM, while locking the full upstream capability map into repository docs.

**Architecture:** Keep the current dashboard pipeline, add a dedicated usage extraction layer per assistant session format, aggregate usage in the snapshot, and surface it in the existing React shell. Treat docs and tests as part of the feature, not follow-up work.

**Tech Stack:** Rust, serde_json, React, Vitest, cargo test, PowerShell verification

---

### Task 1: Write The Master Spec And Plan

**Files:**
- Create: `docs/specs/2026-03-16-upstream-absorption-master-spec.md`
- Create: `docs/plans/2026-03-16-upstream-absorption-master-plan.md`

**Step 1: Write the failing test**

There is no automated test. The failure is repository state: upstream absorption still lacks one source-of-truth spec and an execution plan.

**Step 2: Verify the gap exists**

Run: `rg -n "Upstream Absorption Master" docs/specs docs/plans`
Expected: no master spec or plan exists yet.

**Step 3: Write minimal implementation**

Add the master spec and plan, including:

- researched upstream set
- real absorption ledger
- this tranche scope
- hard acceptance criteria

**Step 4: Verify the docs exist**

Run: `Get-Content docs/specs/2026-03-16-upstream-absorption-master-spec.md`
Expected: the new spec exists and clearly separates absorbed vs researched capabilities.

**Step 5: Commit**

```bash
git add docs/specs/2026-03-16-upstream-absorption-master-spec.md docs/plans/2026-03-16-upstream-absorption-master-plan.md
git commit -m "docs: add upstream absorption master spec"
```

### Task 2: Write Failing Usage Extraction Tests

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing test**

Add tests that expect:

- fixture snapshot emits usage/cost records
- selected session detail exposes usage numbers
- overview renders a usage analytics panel

**Step 2: Run test to verify it fails**

Run: `cargo test fixture_snapshot_includes_usage_analytics -- --exact`
Expected: FAIL because snapshot currently has no usage fields.

Run: `npm --prefix web run test -- app.test.tsx`
Expected: FAIL because the UI currently has no usage panel.

**Step 3: Write minimal implementation**

Only test code and expectations. Do not add production code yet.

**Step 4: Run test to verify it still fails for the expected reason**

Expected: missing `usage` / `usageSummary` fields or missing UI copy.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs web/src/app.test.tsx
git commit -m "test: cover usage analytics snapshot and ui"
```

### Task 3: Implement Rust Usage Extraction And Aggregation

**Files:**
- Create: `src-tauri/src/usage.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `tests/fixtures/codex/...`
- Modify: `tests/fixtures/claude/...`
- Modify: `tests/fixtures/gemini/...`
- Modify: `tests/fixtures/openclaw/...`

**Step 1: Write the failing test**

Use Task 2 failing tests.

**Step 2: Run test to verify it fails**

Run: `cargo test fixture_snapshot_includes_usage_analytics -- --exact`
Expected: FAIL because no extraction exists.

**Step 3: Write minimal implementation**

Add:

- assistant-specific usage extractors for `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw`
- per-session usage summary
- aggregate usage summary by assistant
- snapshot serialization support

Use existing fixture formats or extend fixtures with real-looking usage events based on upstream-documented schemas.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib -- --test-threads=1`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/usage.rs src-tauri/src/lib.rs src-tauri/src/commands/dashboard.rs tests/fixtures
git commit -m "feat: add session usage analytics"
```

### Task 4: Implement Web Usage Surface

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/i18n.tsx`
- Create: `web/src/components/usage-panel.tsx`
- Modify: `web/src/routes/index.tsx`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write the failing test**

Use the web test from Task 2.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- app.test.tsx`
Expected: FAIL because usage copy and panel do not exist.

**Step 3: Write minimal implementation**

Render:

- overview usage panel with assistant totals
- selected session usage detail
- bilingual labels

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/i18n.tsx web/src/components/usage-panel.tsx web/src/routes/index.tsx web/src/components/session-detail.tsx web/src/styles.css web/src/app.test.tsx
git commit -m "feat: surface usage analytics in web ui"
```

### Task 5: Sync Release-Facing Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/research/2026-03-16-full-competitor-gap-analysis.md`

**Step 1: Write the failing test**

There is no automated doc test. The failure is a stale claim gap: docs still say usage/cost analytics is entirely missing.

**Step 2: Verify the gap exists**

Run: `rg -n "token / cost|usage analytics|Tokscale" README.md docs/release docs/research`
Expected: docs do not yet describe the newly absorbed usage capability.

**Step 3: Write minimal implementation**

Update docs to say exactly:

- which assistants now expose usage
- what is still missing
- that pricing sync and broader analytics are still pending

**Step 4: Verify the docs**

Run: `rg -n "usage analytics|token|cost|Tokscale" README.md docs/release docs/research`
Expected: newly absorbed capability is documented consistently.

**Step 5: Commit**

```bash
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/research/2026-03-16-full-competitor-gap-analysis.md
git commit -m "docs: publish usage analytics absorption status"
```

### Task 6: Full Verification And Push

**Files:**
- Review only unless fixes are needed

**Step 1: Run Rust verification**

Run:

```bash
cargo test --lib -- --test-threads=1
cargo test --test cli_snapshot -- --test-threads=1
```

Expected: PASS.

**Step 2: Run web verification**

Run:

```bash
npm --prefix web run test
npm --prefix web run build
```

Expected: PASS.

**Step 3: Run unified verification**

Run:

```bash
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

Expected: PASS.

**Step 4: Push**

Run:

```bash
git push origin feat/usability-clarity
```

Expected: PASS when network is available.

**Step 5: Record outcome**

Document:

- which upstream features are now truly absorbed
- which competitors still lead in search, process control, worktrees, provider governance, and platformization
- which next tranche should follow
