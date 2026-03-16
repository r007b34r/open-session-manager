# Web-First Config Governance Phase 2 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real `Gemini CLI` and `OpenClaw` config governance to OSM, upgrade the config preview surface, and lock the project onto a Web-first future runtime plan.

**Architecture:** Keep the current Rust dashboard and audit pipeline, extend discovery and config auditing for new assistants, then surface the richer records in the existing React config panel. Record the Web-first dual-mode decision in spec/plan instead of mixing it into the same code changes.

**Tech Stack:** Rust, serde_json, TOML, JSON5, React, Vitest, cargo test, PowerShell verification

---

### Task 1: Lock Scope In Spec And Plan

**Files:**
- Create: `docs/specs/2026-03-16-web-first-config-governance-phase2.md`
- Create: `docs/plans/2026-03-16-web-first-config-governance-phase2.md`

**Step 1: Write the failing test**

There is no executable test here. The failure condition is repository state: Phase 2 currently has no execution-ready spec tying Web-first direction to concrete config governance work.

**Step 2: Verify the gap exists**

Run: `rg -n "Web-first|Gemini CLI|OpenClaw config governance" docs/specs docs/plans`
Expected: no Phase 2 spec/plan covering this exact scope.

**Step 3: Write minimal implementation**

Add the new spec and plan with exact scope, non-goals, and acceptance criteria.

**Step 4: Verify the docs exist**

Run: `Get-Content docs/specs/2026-03-16-web-first-config-governance-phase2.md`
Expected: spec exists and captures Web-first runtime decision plus Gemini/OpenClaw config scope.

**Step 5: Commit**

```bash
git add docs/specs/2026-03-16-web-first-config-governance-phase2.md docs/plans/2026-03-16-web-first-config-governance-phase2.md
git commit -m "docs: define web-first config governance phase 2"
```

### Task 2: Write Failing Config Audit Tests

**Files:**
- Create: `tests/fixtures/configs/gemini/.env`
- Create: `tests/fixtures/configs/gemini/settings.json`
- Create: `tests/fixtures/configs/openclaw/openclaw.json`
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing test**

Add tests that expect:

- `audit_config()` supports `gemini-cli`
- `audit_config()` supports `openclaw`
- fixture dashboard snapshot now includes both new config records

**Step 2: Run test to verify it fails**

Run: `cargo test audit::tests -- --nocapture`
Expected: FAIL because `gemini-cli` and `openclaw` are currently unsupported assistants.

**Step 3: Write minimal implementation**

Add realistic Gemini/OpenClaw config fixtures and snapshot expectations.

**Step 4: Run test to verify it passes**

Run: `cargo test audit::tests -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add tests/fixtures/configs src-tauri/src/audit/tests.rs src-tauri/src/commands/dashboard.rs
git commit -m "test: cover gemini and openclaw config audits"
```

### Task 3: Implement Gemini CLI Config Audit

**Files:**
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/audit/config_audit.rs`
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing test**

Use the Gemini audit test from Task 2 as the failing test.

**Step 2: Run test to verify it fails**

Run: `cargo test audits_gemini_config_and_masks_credentials -- --exact`
Expected: FAIL with `unsupported assistant: gemini-cli`.

**Step 3: Write minimal implementation**

Implement:

- discovery for `~/.gemini/settings.json`
- sibling `.env` loading
- provider / model / base URL / auth mode parsing
- official vs proxy classification
- risk detection for missing secrets and custom endpoints

**Step 4: Run test to verify it passes**

Run: `cargo test audits_gemini_config_and_masks_credentials -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/discovery.rs src-tauri/src/audit/config_audit.rs src-tauri/src/audit/tests.rs src-tauri/src/commands/dashboard.rs
git commit -m "feat: add gemini cli config audit"
```

### Task 4: Implement OpenClaw Config Audit

**Files:**
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/audit/config_audit.rs`
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing test**

Use the OpenClaw audit test from Task 2 as the failing test.

**Step 2: Run test to verify it fails**

Run: `cargo test audits_openclaw_config_and_detects_proxy_risks -- --exact`
Expected: FAIL with `unsupported assistant: openclaw`.

**Step 3: Write minimal implementation**

Implement:

- discovery for `~/.openclaw/openclaw.json`
- JSON5 parsing
- provider/base URL/api key/model extraction from `models.providers` and `agents.defaults`
- tools profile and env health signals
- risk detection for dangerous tools profiles and third-party endpoints

**Step 4: Run test to verify it passes**

Run: `cargo test audits_openclaw_config_and_detects_proxy_risks -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/discovery.rs src-tauri/src/audit/config_audit.rs src-tauri/src/audit/tests.rs src-tauri/src/commands/dashboard.rs
git commit -m "feat: add openclaw config audit"
```

### Task 5: Upgrade Config Preview Surface

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/components/config-risk-panel.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing test**

Add a web test that expects the config panel to show a config model field when present.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test`
Expected: FAIL because the current config record shape has no model field and the panel cannot render it.

**Step 3: Write minimal implementation**

Add `model` to the dashboard config record and render it in the config panel with i18n copy.

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs web/src/lib/api.ts web/src/components/config-risk-panel.tsx web/src/lib/i18n.tsx web/src/app.test.tsx
git commit -m "feat: enrich config preview surface"
```

### Task 6: Update Release-Facing Docs And Competitor Gap Trail

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/research/2026-03-16-full-competitor-gap-analysis.md`

**Step 1: Write the failing test**

There is no automated doc test. The failure condition is mismatch: docs still say Gemini/OpenClaw config governance is missing.

**Step 2: Verify the mismatch exists**

Run: `rg -n "Gemini CLI|OpenClaw|配置审计|config audit" README.md docs/release docs/research`
Expected: docs reflect old state and need sync.

**Step 3: Write minimal implementation**

Update support matrix, release notes, README, and competitor gap analysis to distinguish newly absorbed functionality from remaining gaps.

**Step 4: Verify the docs exist**

Run: `rg -n "Gemini CLI|OpenClaw" README.md docs/release docs/research`
Expected: new config governance coverage appears consistently.

**Step 5: Commit**

```bash
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/research/2026-03-16-full-competitor-gap-analysis.md
git commit -m "docs: publish config governance phase 2 status"
```

### Task 7: Full Verification

**Files:**
- Review only unless fixes are needed

**Step 1: Run Rust verification**

Run:

```bash
cargo test --lib -- --test-threads=1
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
git push -u origin feat/usability-clarity
```

**Step 5: Record outcome**

Document exactly:

- what was newly absorbed
- what is still missing
- why Web-first is now the recommended default runtime direction
