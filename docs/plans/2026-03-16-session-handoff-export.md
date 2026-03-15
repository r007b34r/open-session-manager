# Session Handoff Export Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a resume-friendly handoff section to exported Markdown so users can preserve actionable session value before cleanup.

**Architecture:** Extend the existing Rust Markdown exporter with a small handoff derivation layer based on transcript todos, highlights, and current insight summary. Keep the feature purely deterministic and reuse the current export pipeline plus audit trail.

**Tech Stack:** Rust, existing `src-tauri` action exporter, Rust unit tests

---

### Task 1: Add failing export assertions

**Files:**
- Modify: `src-tauri/src/actions/tests.rs`

**Step 1: Write the failing test**

Update the Markdown export tests so they expect:
- `## Session Handoff`
- `Next focus`
- `Open tasks`
- `Completed tasks`
- `Resume cue`

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest`
Expected: FAIL because the handoff section is not rendered yet.

**Step 3: Commit**

Do not commit yet. Continue after green.

### Task 2: Implement deterministic handoff rendering

**Files:**
- Modify: `src-tauri/src/actions/export.rs`

**Step 1: Write minimal implementation**

Add helper functions to derive:
- next focus
- open/completed counts
- resume cue

Render a new `## Session Handoff` section into the existing Markdown template.

**Step 2: Run targeted test to verify it passes**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest`
Expected: PASS

### Task 3: Cover fallback behavior

**Files:**
- Modify: `src-tauri/src/actions/tests.rs`

**Step 1: Write a failing fallback test**

Add a test for a session with no todos and verify the exporter still emits `Session Handoff` using summary fallback.

**Step 2: Run test to verify it fails**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_without_todos_still_builds_session_handoff`
Expected: FAIL until fallback logic is complete.

**Step 3: Extend minimal implementation if needed**

Keep fallback logic deterministic and local to the exporter.

**Step 4: Run both targeted tests**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_with_upstream_style_digest`
Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests::exports_markdown_without_todos_still_builds_session_handoff`
Expected: PASS

### Task 4: Run regression verification

**Files:**
- Modify: none unless regressions are found

**Step 1: Run action test subset**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test actions::tests`
Expected: PASS

**Step 2: Run full verification later with existing project script**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS after the rest of the branch stays green.
