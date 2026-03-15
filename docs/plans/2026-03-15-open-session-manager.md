# open Session Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the first working local-first app that detects Codex, Claude Code, and OpenCode installations on Windows 11/Linux, indexes sessions and configs securely, and supports Markdown export plus safe cleanup workflows.

**Architecture:** Use a Rust core service with an adapter-based ingestion pipeline and a SQLite metadata store, exposed to a React UI through Tauri commands plus a local HTTP/WebSocket server for `serve` mode. Treat Windows native paths and WSL distributions as separate environments merged into one canonical index.

**Tech Stack:** Rust, Tauri, SQLite, React, TypeScript, Vite, TanStack Query, TanStack Router, Vitest, Playwright

---

### Task 1: Repository Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `web/package.json`
- Create: `web/tsconfig.json`
- Create: `web/vite.config.ts`
- Create: `web/src/main.tsx`
- Create: `web/src/app.tsx`
- Test: `cargo test`
- Test: `npm --prefix web run test`

**Step 1: Write the failing test**

Create `src-tauri/src/lib.rs` with a smoke test that expects an app state constructor and a health-check function.

**Step 2: Run test to verify it fails**

Run: `cargo test`
Expected: FAIL because the app state and health-check functions do not exist yet.

**Step 3: Write minimal implementation**

Add the Rust crate layout, define an `AppState`, expose a `health_check()` function, and create the minimal React shell.

**Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: PASS for the smoke test.

**Step 5: Commit**

```bash
git add Cargo.toml src-tauri web
git commit -m "chore: scaffold rust core and web shell"
```

### Task 2: Canonical Domain Model and SQLite Schema

**Files:**
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/session.rs`
- Create: `src-tauri/src/domain/config.rs`
- Create: `src-tauri/src/domain/audit.rs`
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/schema.sql`
- Create: `src-tauri/src/storage/sqlite.rs`
- Test: `src-tauri/src/storage/tests.rs`

**Step 1: Write the failing test**

Add tests that expect the SQLite bootstrap to create tables for installations, sessions, insights, config artifacts, credential artifacts, and audit events.

**Step 2: Run test to verify it fails**

Run: `cargo test storage::`
Expected: FAIL because schema bootstrap and tables are missing.

**Step 3: Write minimal implementation**

Define the canonical structs and create the bootstrap code that initializes the schema.

**Step 4: Run test to verify it passes**

Run: `cargo test storage::`
Expected: PASS and a temporary SQLite DB is created successfully.

**Step 5: Commit**

```bash
git add src-tauri/src/domain src-tauri/src/storage
git commit -m "feat: add canonical domain model and sqlite schema"
```

### Task 3: Discovery Engine and Environment Inventory

**Files:**
- Create: `src-tauri/src/discovery/mod.rs`
- Create: `src-tauri/src/discovery/windows.rs`
- Create: `src-tauri/src/discovery/linux.rs`
- Create: `src-tauri/src/discovery/wsl.rs`
- Create: `src-tauri/src/commands/discovery.rs`
- Test: `src-tauri/src/discovery/tests.rs`

**Step 1: Write the failing test**

Write tests for conventional path resolution:

- Codex user config
- Claude user config
- OpenCode Linux config
- WSL distribution parsing on Windows

**Step 2: Run test to verify it fails**

Run: `cargo test discovery::`
Expected: FAIL because the discovery module does not exist.

**Step 3: Write minimal implementation**

Implement deterministic discovery of known roots using environment variables, home directory resolution, and `wsl.exe -l -q` parsing on Windows.

**Step 4: Run test to verify it passes**

Run: `cargo test discovery::`
Expected: PASS for path resolution and WSL inventory parsing.

**Step 5: Commit**

```bash
git add src-tauri/src/discovery src-tauri/src/commands/discovery.rs
git commit -m "feat: add discovery engine for native and wsl environments"
```

### Task 4: Codex, Claude Code, and OpenCode Adapters

**Files:**
- Create: `src-tauri/src/adapters/mod.rs`
- Create: `src-tauri/src/adapters/traits.rs`
- Create: `src-tauri/src/adapters/codex.rs`
- Create: `src-tauri/src/adapters/claude_code.rs`
- Create: `src-tauri/src/adapters/opencode.rs`
- Test: `src-tauri/src/adapters/tests.rs`
- Test data: `tests/fixtures/codex/`
- Test data: `tests/fixtures/claude/`
- Test data: `tests/fixtures/opencode/`

**Step 1: Write the failing test**

Add fixture-driven tests that expect each adapter to:

- detect at least one session
- parse core metadata
- return a canonical `SessionRecord`

**Step 2: Run test to verify it fails**

Run: `cargo test adapters::`
Expected: FAIL because the adapters and fixtures are not wired up yet.

**Step 3: Write minimal implementation**

Implement the adapter trait and build the three initial parsers:

- Codex JSONL rollout/session parsing
- Claude Code JSONL transcript parsing
- OpenCode metadata/session parsing from its official data roots

**Step 4: Run test to verify it passes**

Run: `cargo test adapters::`
Expected: PASS for all three providers using local fixtures.

**Step 5: Commit**

```bash
git add src-tauri/src/adapters tests/fixtures
git commit -m "feat: add initial codex claude and opencode adapters"
```

### Task 5: Session Insight Pipeline

**Files:**
- Create: `src-tauri/src/insights/mod.rs`
- Create: `src-tauri/src/insights/title.rs`
- Create: `src-tauri/src/insights/progress.rs`
- Create: `src-tauri/src/insights/value.rs`
- Create: `src-tauri/src/insights/garbage.rs`
- Test: `src-tauri/src/insights/tests.rs`

**Step 1: Write the failing test**

Add tests that expect a parsed session fixture to produce:

- a non-empty title
- a progress state
- a value score
- a garbage score

**Step 2: Run test to verify it fails**

Run: `cargo test insights::`
Expected: FAIL because no derivation pipeline exists.

**Step 3: Write minimal implementation**

Build deterministic heuristics from first user goal, assistant summary, TODO/tool events, errors, and last activity timestamps.

**Step 4: Run test to verify it passes**

Run: `cargo test insights::`
Expected: PASS with stable derived outputs for fixtures.

**Step 5: Commit**

```bash
git add src-tauri/src/insights
git commit -m "feat: derive titles progress and retention scores"
```

### Task 6: Config and Credential Auditor

**Files:**
- Create: `src-tauri/src/audit/config_audit.rs`
- Create: `src-tauri/src/audit/credential_audit.rs`
- Create: `src-tauri/src/audit/redaction.rs`
- Create: `src-tauri/src/commands/configs.rs`
- Test: `src-tauri/src/audit/tests.rs`
- Test data: `tests/fixtures/configs/`

**Step 1: Write the failing test**

Write tests expecting the auditor to:

- parse assistant config layers
- detect non-official base URLs
- mask secrets by default
- emit risk flags for dangerous settings

**Step 2: Run test to verify it fails**

Run: `cargo test audit::`
Expected: FAIL because the audit pipeline does not exist.

**Step 3: Write minimal implementation**

Implement parsers for known config formats, masked secret fingerprints, and relay detection using allowlists plus endpoint heuristics.

**Step 4: Run test to verify it passes**

Run: `cargo test audit::`
Expected: PASS with masked outputs and stable risk flags.

**Step 5: Commit**

```bash
git add src-tauri/src/audit src-tauri/src/commands/configs.rs tests/fixtures/configs
git commit -m "feat: add config and credential auditing"
```

### Task 7: Markdown Export, Soft Delete, and Audit Log

**Files:**
- Create: `src-tauri/src/actions/mod.rs`
- Create: `src-tauri/src/actions/export.rs`
- Create: `src-tauri/src/actions/delete.rs`
- Create: `src-tauri/src/actions/restore.rs`
- Create: `src-tauri/src/commands/actions.rs`
- Test: `src-tauri/src/actions/tests.rs`

**Step 1: Write the failing test**

Add tests that expect:

- Markdown export to write a file with frontmatter and key sections
- delete to move a session into quarantine instead of hard deleting
- restore to put the session back
- all actions to create audit rows

**Step 2: Run test to verify it fails**

Run: `cargo test actions::`
Expected: FAIL because no export/delete/restore workflow exists.

**Step 3: Write minimal implementation**

Implement the action engine with manifest files, quarantine paths, and audit inserts.

**Step 4: Run test to verify it passes**

Run: `cargo test actions::`
Expected: PASS and action side effects are verified in temp directories.

**Step 5: Commit**

```bash
git add src-tauri/src/actions src-tauri/src/commands/actions.rs
git commit -m "feat: add markdown export and safe cleanup workflows"
```

### Task 8: React UI for Overview, Explorer, Detail, and Config Center

**Files:**
- Create: `web/src/routes/__root.tsx`
- Create: `web/src/routes/index.tsx`
- Create: `web/src/routes/sessions.tsx`
- Create: `web/src/routes/sessions.$id.tsx`
- Create: `web/src/routes/configs.tsx`
- Create: `web/src/routes/audit.tsx`
- Create: `web/src/components/session-table.tsx`
- Create: `web/src/components/session-detail.tsx`
- Create: `web/src/components/config-risk-panel.tsx`
- Create: `web/src/lib/api.ts`
- Test: `web/src/components/session-table.test.tsx`
- Test: `web/src/components/config-risk-panel.test.tsx`

**Step 1: Write the failing test**

Add component tests that expect:

- session rows to render title, assistant, progress, and last activity
- config risk panel to render masked values and risk badges

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test`
Expected: FAIL because the routes and components do not exist.

**Step 3: Write minimal implementation**

Build the first UI slices:

- Overview dashboard
- Session explorer
- Session detail
- Config center
- Audit center

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test`
Expected: PASS for the initial UI behavior.

**Step 5: Commit**

```bash
git add web/src
git commit -m "feat: add initial governance dashboard ui"
```

### Task 9: End-to-End Verification

**Files:**
- Create: `tests/e2e/session-governance.spec.ts`
- Modify: `web/package.json`
- Modify: `README.md`

**Step 1: Write the failing test**

Create an end-to-end flow that verifies:

- fixtures are indexed
- a session can be exported
- a session can be soft-deleted
- the config center shows a masked risky endpoint

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run e2e`
Expected: FAIL because the integration flow is incomplete.

**Step 3: Write minimal implementation**

Finish the API plumbing, seed a fixture workspace for development, and document how to run the app locally.

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run e2e`
Expected: PASS for the core user journey.

**Step 5: Commit**

```bash
git add tests/e2e web/package.json README.md
git commit -m "test: verify export cleanup and config risk flow"
```

Plan complete and saved to `docs/plans/2026-03-15-open-session-manager.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
