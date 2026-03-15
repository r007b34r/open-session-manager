# Usability and Clarity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix session selection reliability, expose export destinations and export settings, add light/dark theme switching, make upstream absorption visible in the product UI, and rewrite the homepage copy so it reads like a product instead of generated filler.

**Architecture:** Keep the existing Rust core and React/Tauri shell, but add one thin preferences layer shared by desktop commands and the UI. Treat the work as five linked product tasks: harden the session workspace interaction model first, then expose file destinations and settings, then layer theme state and product-proof sections on top of the existing overview shell, and finally rewrite the release-facing copy with less marketing filler and more concrete information.

**Tech Stack:** Rust, Tauri, React, TypeScript, Vitest, Playwright, Markdown, local JSON preferences

---

### Task 1: Make Session Selection Work Like a Real Workspace

**Files:**
- Modify: `web/src/components/session-table.tsx`
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/app.tsx`
- Modify: `web/src/styles.css`
- Test: `web/src/components/session-table.test.tsx`
- Test: `web/src/app.test.tsx`
- Test: `tests/e2e/open-session-manager.spec.ts`

**Step 1: Write the failing tests**

Add tests that expect:

- clicking anywhere on a session row selects that session
- the selected row stays visually distinct
- real route state updates to the clicked session instead of silently keeping the first session

**Step 2: Run tests to verify they fail**

Run:

```bash
npm --prefix web run test -- src/components/session-table.test.tsx src/app.test.tsx
npm --prefix web run e2e -- --grep "session row selection"
```

Expected: FAIL because the current table only binds selection to a small button hit area instead of the whole row workspace interaction.

**Step 3: Write minimal implementation**

Change the session table interaction model so:

- the whole row is a reliable click target
- keyboard focus still works
- selected state is driven from one source of truth
- route changes and list state stay in sync

**Step 4: Run tests to verify they pass**

Run:

```bash
npm --prefix web run test -- src/components/session-table.test.tsx src/app.test.tsx
npm --prefix web run e2e -- --grep "session row selection"
```

Expected: PASS with the clicked session becoming active in both UI state and route state.

**Step 5: Commit**

```bash
git add web/src/components/session-table.tsx web/src/routes/sessions.tsx web/src/app.tsx web/src/styles.css web/src/components/session-table.test.tsx web/src/app.test.tsx tests/e2e/open-session-manager.spec.ts
git commit -m "fix: make session row selection reliable"
```

### Task 2: Show Export Destinations and Add Export Path Preferences

**Files:**
- Modify: `src-tauri/src/desktop.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/actions/export.rs`
- Create: `src-tauri/src/preferences.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/routes/index.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`
- Test: `src-tauri/tests/cli_snapshot.rs`
- Test: `web/src/lib/api.test.ts`
- Test: `web/src/app.test.tsx`

**Step 1: Write the failing tests**

Add tests that expect:

- dashboard data includes export root, audit db path, and quarantine root
- exporting a session returns or exposes the written Markdown path
- the UI can display the current export location and save a user-defined export root

**Step 2: Run tests to verify they fail**

Run:

```bash
C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot
npm --prefix web run test -- src/lib/api.test.ts src/app.test.tsx
```

Expected: FAIL because the current UI neither shows export destinations nor exposes a writable export-root preference.

**Step 3: Write minimal implementation**

Add a preferences layer that:

- persists an optional export root override in local app data
- includes runtime paths in the dashboard snapshot
- shows the current export destination after export
- provides a simple settings surface for the export root

**Step 4: Run tests to verify they pass**

Run:

```bash
C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot
npm --prefix web run test -- src/lib/api.test.ts src/app.test.tsx
```

Expected: PASS with visible runtime paths and a working export destination preference.

**Step 5: Commit**

```bash
git add src-tauri/src/desktop.rs src-tauri/src/commands/dashboard.rs src-tauri/src/actions/export.rs src-tauri/src/preferences.rs src-tauri/src/lib.rs web/src/lib/api.ts web/src/routes/index.tsx web/src/lib/i18n.tsx web/src/styles.css src-tauri/tests/cli_snapshot.rs web/src/lib/api.test.ts web/src/app.test.tsx
git commit -m "feat: expose export destinations and preferences"
```

### Task 3: Add Theme Mode With System Default and Manual Override

**Files:**
- Modify: `web/src/routes/__root.tsx`
- Modify: `web/src/app.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Create: `web/src/lib/theme.ts`
- Modify: `web/src/styles.css`
- Test: `web/src/app.test.tsx`

**Step 1: Write the failing tests**

Add tests that expect:

- the app defaults to the system theme
- the user can manually switch light and dark mode
- the choice persists across reloads

**Step 2: Run tests to verify they fail**

Run:

```bash
npm --prefix web run test -- src/app.test.tsx
```

Expected: FAIL because theme state and theme controls do not exist yet.

**Step 3: Write minimal implementation**

Add theme state that:

- reads the system preference by default
- persists a manual override
- updates CSS variables and `color-scheme`
- exposes a compact toggle in the root shell

**Step 4: Run tests to verify they pass**

Run:

```bash
npm --prefix web run test -- src/app.test.tsx
```

Expected: PASS with deterministic theme switching and persistence.

**Step 5: Commit**

```bash
git add web/src/routes/__root.tsx web/src/app.tsx web/src/lib/i18n.tsx web/src/lib/theme.ts web/src/styles.css web/src/app.test.tsx
git commit -m "feat: add theme switching with persisted override"
```

### Task 4: Make Upstream Absorption Visible Inside the Product

**Files:**
- Modify: `web/src/routes/index.tsx`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`
- Modify: `docs/release/github-release-notes.md`
- Test: `web/src/app.test.tsx`

**Step 1: Write the failing tests**

Add tests that expect:

- the overview page explicitly lists absorbed upstream capabilities
- the UI distinguishes adopted capabilities from screened-only research
- the product shows a direct, concrete “what was absorbed” explanation instead of hiding it in docs

**Step 2: Run tests to verify they fail**

Run:

```bash
npm --prefix web run test -- src/app.test.tsx
```

Expected: FAIL because current upstream adoption evidence only exists in docs and release notes, not in the app.

**Step 3: Write minimal implementation**

Add an overview section that:

- highlights adopted upstream features
- links the feature proof to current OSM behavior
- separates adopted implementations from screened inspirations

**Step 4: Run tests to verify they pass**

Run:

```bash
npm --prefix web run test -- src/app.test.tsx
```

Expected: PASS with visible adoption proof in the product shell.

**Step 5: Commit**

```bash
git add web/src/routes/index.tsx web/src/lib/api.ts web/src/lib/i18n.tsx web/src/styles.css docs/release/github-release-notes.md web/src/app.test.tsx
git commit -m "feat: surface upstream adoption in overview"
```

### Task 5: Rewrite Homepage and Release Copy to Remove AI-Smelling Filler

**Files:**
- Modify: `README.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `web/src/lib/i18n.tsx`
- Test: manual doc review

**Step 1: Write the failing test**

Review the current copy and note the places where it sounds inflated, repetitive, or vague instead of concrete.

**Step 2: Run test to verify it fails**

Run:

```bash
rg -n "本地优先|治理平台|公开预览|已吸收|当前已实现" README.md docs/release/github-release-notes.md web/src/lib/i18n.tsx
```

Expected: existing copy still contains too much high-level framing and not enough direct user-facing explanation.

**Step 3: Write minimal implementation**

Use `humanizer-zh` style edits to:

- remove inflated positioning language
- replace abstract claims with concrete actions and boundaries
- reduce duplicated release framing
- make the homepage read like a tool description, not generated marketing

**Step 4: Run test to verify it passes**

Run:

```bash
rg -n "导出路径|主题|已吸收|当前边界|发布后优先项" README.md docs/release/github-release-notes.md docs/release/support-matrix.md
```

Expected: the docs emphasize concrete behavior, visible settings, boundaries, and adoption proof.

**Step 5: Commit**

```bash
git add README.md docs/release/github-release-notes.md docs/release/support-matrix.md web/src/lib/i18n.tsx
git commit -m "docs: rewrite homepage and release copy with concrete language"
```
