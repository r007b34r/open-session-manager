# Upstream Absorption Phase 1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real `Gemini CLI`, `GitHub Copilot CLI`, and `Factory Droid` session support to OSM with TDD, dashboard integration, tests, and release-facing documentation.

**Architecture:** Keep the existing Rust domain model stable and extend the adapter pipeline, discovery roots, dashboard narrative extraction, and transcript digest branches. Each assistant is introduced through fixture-first tests, then minimal parsing code, then snapshot-level verification.

**Tech Stack:** Rust, serde_json, Tauri core, cargo test, Markdown docs, git

---

### Task 1: Lock Phase 1 Scope In Spec And Research Trail

**Files:**
- Create: `docs/specs/2026-03-16-multi-assistant-absorption-phase1.md`
- Create: `docs/plans/2026-03-16-upstream-absorption-phase1.md`
- Modify: `docs/research/upstreams/index.md`

**Step 1: Write the failing test**

There is no executable test here. The failure condition is repository state: the repo currently has no concrete Phase 1 spec covering real multi-assistant absorption.

**Step 2: Verify the gap exists**

Run: `rg -n "Absorption Phase 1|GitHub Copilot CLI|Factory Droid" docs`
Expected: existing docs mention research targets, but no single execution-ready Phase 1 spec and plan.

**Step 3: Write minimal implementation**

Add a spec that records upstream evidence, scope, non-goals, architecture, and acceptance criteria. Add this implementation plan with exact file paths and commands.

**Step 4: Verify the docs exist**

Run: `Get-Content docs/specs/2026-03-16-multi-assistant-absorption-phase1.md`
Expected: file exists and captures scope for Gemini, Copilot, and Droid.

**Step 5: Commit**

```bash
git add docs/specs/2026-03-16-multi-assistant-absorption-phase1.md docs/plans/2026-03-16-upstream-absorption-phase1.md docs/research/upstreams/index.md
git commit -m "docs: define multi-assistant absorption phase 1"
```

### Task 2: Write Failing Adapter Tests And Fixtures

**Files:**
- Create: `tests/fixtures/gemini/tmp/project-hash-a/chats/session-gemini-demo.json`
- Create: `tests/fixtures/copilot/session-state/copilot-ses-1.jsonl`
- Create: `tests/fixtures/factory/sessions/project-a/droid-session-1.jsonl`
- Create: `tests/fixtures/factory/projects/project-a/stream-session-1.jsonl`
- Modify: `src-tauri/src/adapters/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing test**

Add tests that expect:

- `Gemini CLI` fixture discovery and parsing works
- `GitHub Copilot CLI` fixture discovery and parsing works
- `Factory Droid` fixture discovery and parsing works for both session-store and stream-json variants
- fixture dashboard snapshot session count becomes `6`
- snapshot output includes new assistant names and summaries

**Step 2: Run test to verify it fails**

Run: `cargo test adapters::tests -- --nocapture`
Expected: FAIL because the new adapters and fixtures are not wired yet.

Run: `cargo test commands::dashboard::tests::fixture_snapshot_includes_transcript_digest -- --exact`
Expected: FAIL because fixture snapshot still only knows three assistants.

**Step 3: Write minimal implementation**

Create minimal fixtures that reflect upstream structures and adjust snapshot expectations to the new session count and assistant coverage.

**Step 4: Run test to verify it passes**

Run: `cargo test adapters::tests -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/fixtures src-tauri/src/adapters/tests.rs src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs
git commit -m "test: cover phase 1 multi-assistant fixtures"
```

### Task 3: Implement Gemini CLI Adapter And Dashboard Chain

**Files:**
- Create: `src-tauri/src/adapters/gemini_cli.rs`
- Modify: `src-tauri/src/adapters/mod.rs`
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/transcript/mod.rs`

**Step 1: Write the failing test**

Use the Gemini fixture test from Task 2 as the failing test.

**Step 2: Run test to verify it fails**

Run: `cargo test gemini_adapter_discovers_and_parses_fixture -- --exact`
Expected: FAIL with missing module / unsupported assistant errors.

**Step 3: Write minimal implementation**

Implement:

- discovery of `session-*.json`
- JSON root-shape tolerant parsing
- user/assistant/tool-call counting
- dashboard narrative extraction for first user goal and latest assistant output
- transcript digest extraction for highlights and todo-safe summaries

**Step 4: Run test to verify it passes**

Run: `cargo test gemini_adapter_discovers_and_parses_fixture -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/adapters/gemini_cli.rs src-tauri/src/adapters/mod.rs src-tauri/src/commands/discovery.rs src-tauri/src/commands/dashboard.rs src-tauri/src/transcript/mod.rs tests/fixtures/gemini src-tauri/src/adapters/tests.rs
git commit -m "feat: add gemini cli session support"
```

### Task 4: Implement GitHub Copilot CLI Adapter And Dashboard Chain

**Files:**
- Create: `src-tauri/src/adapters/copilot_cli.rs`
- Modify: `src-tauri/src/adapters/mod.rs`
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/transcript/mod.rs`

**Step 1: Write the failing test**

Use the Copilot fixture test from Task 2 as the failing test.

**Step 2: Run test to verify it fails**

Run: `cargo test copilot_adapter_discovers_and_parses_fixture -- --exact`
Expected: FAIL with missing module / unsupported assistant errors.

**Step 3: Write minimal implementation**

Implement:

- discovery of `.copilot/session-state/*.jsonl`
- event-envelope parsing
- assistant and tool execution counting
- dashboard narrative extraction
- transcript highlights for user, assistant, and tool completion output

**Step 4: Run test to verify it passes**

Run: `cargo test copilot_adapter_discovers_and_parses_fixture -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/adapters/copilot_cli.rs src-tauri/src/adapters/mod.rs src-tauri/src/commands/discovery.rs src-tauri/src/commands/dashboard.rs src-tauri/src/transcript/mod.rs tests/fixtures/copilot src-tauri/src/adapters/tests.rs
git commit -m "feat: add github copilot cli session support"
```

### Task 5: Implement Factory Droid Adapter And Dashboard Chain

**Files:**
- Create: `src-tauri/src/adapters/factory_droid.rs`
- Modify: `src-tauri/src/adapters/mod.rs`
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/transcript/mod.rs`

**Step 1: Write the failing test**

Use the Droid fixture test from Task 2 as the failing test.

**Step 2: Run test to verify it fails**

Run: `cargo test droid_adapter_discovers_and_parses_fixture -- --exact`
Expected: FAIL with missing module / unsupported assistant errors.

**Step 3: Write minimal implementation**

Implement:

- discovery of both session-store and stream-json roots
- a minimal stream-json recognizer
- session-store parser
- stream-json parser
- dashboard narrative extraction for both dialects
- transcript highlights for both dialects

**Step 4: Run test to verify it passes**

Run: `cargo test droid_adapter_discovers_and_parses_fixture -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/adapters/factory_droid.rs src-tauri/src/adapters/mod.rs src-tauri/src/commands/discovery.rs src-tauri/src/commands/dashboard.rs src-tauri/src/transcript/mod.rs tests/fixtures/factory src-tauri/src/adapters/tests.rs
git commit -m "feat: add factory droid session support"
```

### Task 6: Verify Snapshot And Regression Surface

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing test**

Extend snapshot assertions to require:

- six fixture sessions
- assistant coverage for Gemini, Copilot, and Droid
- real summaries extracted from new assistants

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_snapshot snapshot_command_emits_real_dashboard_json_from_fixtures -- --exact`
Expected: FAIL because fixture count and content are still old.

**Step 3: Write minimal implementation**

Update fixture snapshot roots, dashboard narrative branches, and CLI snapshot assertions.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_snapshot snapshot_command_emits_real_dashboard_json_from_fixtures -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs
git commit -m "test: verify phase 1 adapters in dashboard snapshot"
```

### Task 7: Update Release-Facing Docs And Governance Trail

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/research/upstreams/index.md`
- Modify: `docs/release/open-source-attribution.md`
- Modify: `third_party/upstreams/catalog.json`
- Modify: `third_party/upstreams/intake-manifest.json`

**Step 1: Write the failing test**

There is no automated doc test today. The failure condition is mismatch: docs still claim only three assistants are supported.

**Step 2: Verify the mismatch exists**

Run: `rg -n "Codex.*Claude Code.*OpenCode|已支持的助手" README.md docs/release`
Expected: docs still reflect the old support surface.

**Step 3: Write minimal implementation**

Update support matrix, release notes, README, and upstream attribution to describe the newly integrated assistants and the upstream sources that influenced them.

**Step 4: Verify the docs exist**

Run: `rg -n "Gemini CLI|GitHub Copilot CLI|Factory Droid" README.md docs/release docs/research`
Expected: new support surface appears consistently.

**Step 5: Commit**

```bash
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/research/upstreams/index.md docs/release/open-source-attribution.md third_party/upstreams/catalog.json third_party/upstreams/intake-manifest.json
git commit -m "docs: publish phase 1 multi-assistant release notes"
```

### Task 8: Full Verification, Audit, And Push

**Files:**
- Review only unless fixes are needed

**Step 1: Run verification**

Run:

```bash
cargo test --lib
cargo test --test cli_snapshot
npm --prefix web run test
npm --prefix web run build
```

Expected: PASS. If the web layer is unaffected, failures still block completion and must be investigated.

**Step 2: Run code audit**

Perform a local review focused on:

- adapter false positives
- unsupported assistant branches
- narrative gaps that degrade cleanup value
- new regressions in snapshot CLI

**Step 3: Fix minimal defects**

Only fix root causes revealed by verification or audit.

**Step 4: Push**

Run:

```bash
git push -u origin feat/usability-clarity
```

**Step 5: Record outcome**

Document:

- passed verification commands
- remaining risks
- exact support surface after Phase 1
