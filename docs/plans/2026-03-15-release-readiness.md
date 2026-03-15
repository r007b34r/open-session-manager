# Release Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the current prototype into a release-track local-first app that reads real local assistant data, exposes it safely to the UI, and has reproducible verification on Windows 11 and Linux.

**Architecture:** Keep the Rust core as the source of truth for discovery, parsing, insight generation, config auditing, export, quarantine, restore, and audit logging. Replace the front-end fixture-first path with a real snapshot contract emitted by the Rust core, then bridge that contract into desktop and browser-facing development flows.

**Tech Stack:** Rust, SQLite, React, TypeScript, Vite, Vitest, Playwright, PowerShell, Node.js

---

### Task 1: Restore Release-Grade Verification Entry Points

**Files:**
- Create: `scripts/verify.ps1`
- Modify: `README.md`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe test`
- Test: `npm --prefix web run test`
- Test: `npm --prefix web run build`
- Test: `npm --prefix web run e2e`

**Step 1: Write the failing test**

Create a verification script invocation in docs and run it before the script exists.

**Step 2: Run test to verify it fails**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: FAIL because the script does not exist yet.

**Step 3: Write minimal implementation**

Add a verification script that:

- resolves `cargo` from PATH or common Windows home locations
- runs Rust tests
- runs web tests, build, and Playwright
- exits non-zero on the first failure

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS with all verifications executed in order.

**Step 5: Commit**

```bash
git add scripts/verify.ps1 README.md
git commit -m "chore: add release verification entrypoint"
```

### Task 2: Real Local Snapshot Contract

**Files:**
- Create: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/discovery/mod.rs`
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/adapters/traits.rs`
- Modify: `src-tauri/src/adapters/codex.rs`
- Modify: `src-tauri/src/adapters/claude_code.rs`
- Modify: `src-tauri/src/adapters/opencode.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing test**

Add an integration test that runs the Rust binary with a fixture snapshot command and expects:

- three parsed sessions
- three parsed config entries
- masked credentials
- transcript-derived title and summary fields

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot`
Expected: FAIL because the CLI snapshot command and dashboard contract do not exist.

**Step 3: Write minimal implementation**

Implement a serializable dashboard snapshot builder that:

- discovers real config and session roots
- extracts the first user goal and last assistant message from supported transcripts
- derives title, progress, value, risk, and artifact summaries
- emits JSON from the CLI for both local and fixture-backed runs

**Step 4: Run test to verify it passes**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot`
Expected: PASS with deterministic JSON output for fixture input.

**Step 5: Commit**

```bash
git add src-tauri/src/commands src-tauri/src/discovery src-tauri/src/adapters src-tauri/src/main.rs src-tauri/tests/cli_snapshot.rs
git commit -m "feat: add real dashboard snapshot contract"
```

### Task 3: Web Snapshot Loader and Fixture Fallback

**Files:**
- Create: `web/src/lib/api.test.ts`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/app.tsx`
- Modify: `README.md`

**Step 1: Write the failing test**

Add tests that expect `fetchDashboardSnapshot()` to:

- prefer a real JSON snapshot endpoint when available
- fall back to typed fixtures only when the real endpoint is unavailable

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- src/lib/api.test.ts`
Expected: FAIL because the loader currently only returns hard-coded fixtures.

**Step 3: Write minimal implementation**

Update the loader so the web app first requests a generated or served snapshot, validates the shape, and only then falls back to the embedded demo snapshot.

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- src/lib/api.test.ts`
Expected: PASS for both the real-data path and the fixture fallback path.

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/api.test.ts web/src/app.tsx README.md
git commit -m "feat: load real snapshots before falling back to fixtures"
```

### Task 4: Desktop Bridge and Packaging Skeleton

**Files:**
- Create: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `web/package.json`
- Modify: `README.md`
- Test: `npm --prefix web run build`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe test`

**Step 1: Write the failing test**

Attempt to build or inspect a desktop bridge configuration that does not exist.

**Step 2: Run test to verify it fails**

Run: `Test-Path src-tauri/tauri.conf.json`
Expected: FAIL because the Tauri config is missing.

**Step 3: Write minimal implementation**

Add the first desktop packaging skeleton:

- Tauri config
- Rust and front-end package metadata
- documented dev/build entry points

**Step 4: Run test to verify it passes**

Run: `Test-Path src-tauri/tauri.conf.json`
Expected: PASS and the config can be inspected locally.

**Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/Cargo.toml web/package.json README.md
git commit -m "chore: add desktop packaging skeleton"
```

### Task 5: Safe Cleanup Guardrails and Audit Persistence

**Files:**
- Modify: `src-tauri/src/actions/export.rs`
- Modify: `src-tauri/src/actions/delete.rs`
- Modify: `src-tauri/src/actions/restore.rs`
- Modify: `src-tauri/src/storage/sqlite.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/lib/api.ts`

**Step 1: Write the failing test**

Add tests expecting destructive actions to require export-or-confirm sequencing and to persist audit events into a reusable local database file.

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::`
Expected: FAIL because the new guardrails and persistence rules do not exist yet.

**Step 3: Write minimal implementation**

Enforce:

- explicit confirmation before quarantine
- export-first workflow metadata
- persistent audit database location instead of temporary-only flow

**Step 4: Run test to verify it passes**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::`
Expected: PASS with durable audit evidence.

**Step 5: Commit**

```bash
git add src-tauri/src/actions src-tauri/src/storage/sqlite.rs web/src/components/session-detail.tsx web/src/lib/api.ts
git commit -m "feat: harden cleanup guardrails and audit persistence"
```

### Task 6: CI, Support Matrix, and Release Docs

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `docs/release/support-matrix.md`
- Modify: `README.md`
- Modify: `docs/plans/2026-03-15-open-session-manager-design.md`

**Step 1: Write the failing test**

Verify that the repository has no CI workflow or support matrix.

**Step 2: Run test to verify it fails**

Run: `rg --files .github docs/release`
Expected: FAIL because those release assets do not exist.

**Step 3: Write minimal implementation**

Add:

- CI for Rust and web verification
- supported assistant/platform/version matrix
- release notes prerequisites and known limitations

**Step 4: Run test to verify it passes**

Run: `rg --files .github docs/release`
Expected: PASS and the release assets are present.

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml docs/release/support-matrix.md README.md docs/plans/2026-03-15-open-session-manager-design.md
git commit -m "docs: add release pipeline and support matrix"
```
