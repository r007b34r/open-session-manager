# OSM Audit And UI Evolution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Produce a release-grade audit of the current codebase, harden the Windows desktop launch behavior, and upgrade the Sessions workspace into a stable, responsive, bilingual control surface.

**Architecture:** Keep the Rust core as the source of truth for discovery, parsing, insight generation, export, quarantine, and audit logging. Improve the desktop shell at the binary boundary, then tighten the React workspace so selection, filtering, and detail rendering stay consistent across browser, Tauri, and release builds.

**Tech Stack:** Rust, Tauri v2, React 19, TypeScript, Vite, Vitest, Playwright, PowerShell

---

### Task 1: Audit The Current Release Surface

**Files:**
- Create: `docs/research/2026-03-15-osm-audit-and-roadmap.md`
- Modify: `docs/research/2026-03-15-agent-session-landscape.md`
- Test: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
- Test: `C:\Users\Max\.cargo\bin\cargo.exe clippy --all-targets --all-features -- -D warnings`

**Step 1: Write the failing test**

Run the current verification entrypoints and record the gaps that block a release-grade audit conclusion.

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe clippy --all-targets --all-features -- -D warnings`
Expected: FAIL because the codebase still contains lint-level design and maintainability issues.

**Step 3: Write minimal implementation**

Document:

- verified code audit findings with severity
- local verification status and release blockers
- direct competitor comparison, integration candidates, and license constraints
- a phased roadmap aimed at beating current open-source tools on Windows/Linux governance

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS or produce a concrete blocker list with evidence for any remaining failure.

**Step 5: Commit**

```bash
git add docs/research/2026-03-15-agent-session-landscape.md docs/research/2026-03-15-osm-audit-and-roadmap.md
git commit -m "docs: add audit findings and evolution roadmap"
```

### Task 2: Prevent Console Window On Windows Release Builds

**Files:**
- Modify: `src-tauri/src/main.rs`
- Test: `npm --prefix web run tauri:build`

**Step 1: Write the failing test**

Add a release-build verification step that inspects the Windows executable subsystem and expects a GUI subsystem for the packaged desktop binary.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run tauri:build`
Expected: current release build still produces a console-subsystem executable.

**Step 3: Write minimal implementation**

Add the Windows GUI subsystem attribute at the binary entrypoint without changing snapshot CLI behavior.

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run tauri:build`
Expected: PASS and the produced Windows binary launches without a terminal window in release mode.

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "fix: suppress console window for Windows release builds"
```

### Task 3: Rebuild Session Selection Around A Stable Workspace Model

**Files:**
- Modify: `web/src/app.test.tsx`
- Modify: `web/src/components/session-table.test.tsx`
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/components/session-table.tsx`
- Modify: `web/src/app.tsx`

**Step 1: Write the failing test**

Add tests that expect:

- clicking a session row updates the selected detail pane
- the selected detail pane stays aligned with the visible filtered list
- empty search results no longer show a mismatched stale detail pane

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- src/app.test.tsx src/components/session-table.test.tsx`
Expected: FAIL because selection still depends on link-only navigation and filtered state can drift from the detail pane.

**Step 3: Write minimal implementation**

Refactor the Sessions route into an explicit workspace model with:

- row/button selection callbacks
- hash synchronization for deep links
- filtered-list-aware detail selection
- better empty-state handling

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- src/app.test.tsx src/components/session-table.test.tsx`
Expected: PASS with deterministic selection behavior.

**Step 5: Commit**

```bash
git add web/src/app.tsx web/src/app.test.tsx web/src/routes/sessions.tsx web/src/components/session-table.tsx web/src/components/session-table.test.tsx
git commit -m "fix: stabilize session workspace selection flow"
```

### Task 4: Deeply Refresh The Sessions UI For Responsive Desktop Use

**Files:**
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/styles.css`
- Modify: `tests/e2e/open-session-manager.spec.ts`
- Test: `npm --prefix web run test`
- Test: `npm --prefix web run build`
- Test: `npm --prefix web run e2e`

**Step 1: Write the failing test**

Add end-to-end expectations for selecting different sessions and keeping the detail panel usable under smaller viewport widths.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run e2e`
Expected: FAIL because the current layout stretches the detail panel and does not guarantee responsive selection flows.

**Step 3: Write minimal implementation**

Upgrade the UI with:

- a non-stretched split workspace
- sticky detail panel behavior on wide screens
- bounded scrolling for long transcript metadata
- adaptive stacking on narrower windows
- stronger visual grouping for summary, signals, artifacts, and risk surfaces

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run e2e`
Expected: PASS with stable selection and usable layout across viewports.

**Step 5: Commit**

```bash
git add web/src/components/session-detail.tsx web/src/styles.css tests/e2e/open-session-manager.spec.ts
git commit -m "feat: refresh session workspace layout and responsive behavior"
```

### Task 5: Verify Release Readiness And Attribution Surface

**Files:**
- Modify: `README.md`
- Modify: `docs/release/github-release-notes.md`
- Test: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
- Test: `npm --prefix web run tauri:build`

**Step 1: Write the failing test**

Review the release-facing docs and verify they do not yet reflect the new audit conclusion, acknowledgements, and desktop/UI fixes.

**Step 2: Run test to verify it fails**

Run: `rg -n "Acknowledg|致谢|感谢|competitor|竞品" README.md docs/release/github-release-notes.md`
Expected: FAIL because competitor incubation credits and updated release guidance are incomplete.

**Step 3: Write minimal implementation**

Update release-facing docs with:

- polished product positioning
- clear local usage guidance
- open-source acknowledgements
- what was adopted directly vs only referenced

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS with current docs and implementation aligned.

**Step 5: Commit**

```bash
git add README.md docs/release/github-release-notes.md
git commit -m "docs: update release positioning and acknowledgements"
```
