# Open Session Manager Release Publish Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finalize the first public-facing release assets for open Session Manager, record the work in local git, and publish the repository state and release materials to GitHub.

**Architecture:** Keep the existing Rust core, React/Tauri desktop shell, and upstream-intake assets unchanged unless release-facing metadata or copy needs to be clarified. Treat this task as a release-packaging pass: align versions, homepage copy, release notes, support statements, attribution, and GitHub publishing metadata around the features already implemented.

**Tech Stack:** Git, GitHub CLI, Rust, React, Tauri, Markdown, PowerShell

---

### Task 1: Audit Current Release-Facing Assets

**Files:**
- Modify: `docs/plans/2026-03-15-release-publish.md`
- Inspect: `README.md`
- Inspect: `docs/release/github-release-notes.md`
- Inspect: `docs/release/support-matrix.md`
- Inspect: `docs/release/open-source-attribution.md`
- Inspect: `src-tauri/Cargo.toml`
- Inspect: `web/package.json`
- Inspect: `src-tauri/tauri.conf.json`

**Step 1: Write the failing test**

List the current release-facing assets and note the mismatches in versioning, positioning, and scope.

**Step 2: Run test to verify it fails**

Run: `git -C . status --short`
Expected: Existing release-facing files still need alignment.

**Step 3: Write minimal implementation**

Produce a concrete list of:

- absorbed upstream capabilities
- currently implemented OSM capabilities
- required homepage and release-note changes
- required version and packaging metadata changes

**Step 4: Run test to verify it passes**

Run: `rg -n "当前能力|本版重点|开源致谢|支持矩阵" README.md docs/release/github-release-notes.md docs/release/support-matrix.md`
Expected: Release-facing sections are identifiable and ready to edit.

**Step 5: Commit**

```bash
git add docs/plans/2026-03-15-release-publish.md
git commit -m "docs: add release publishing plan"
```

### Task 2: Align Public Release Copy and Metadata

**Files:**
- Modify: `README.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/open-source-attribution.md`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `web/package.json`

**Step 1: Write the failing test**

Identify inconsistent project naming, outdated version numbers, incomplete absorbed-feature descriptions, or unclear public limitations.

**Step 2: Run test to verify it fails**

Run: `rg -n "\"version\"|version =|open Session Manager|OSM|吸收|限制" README.md docs/release src-tauri/Cargo.toml src-tauri/tauri.conf.json web/package.json`
Expected: Existing release copy and metadata are incomplete or inconsistent.

**Step 3: Write minimal implementation**

Update release-facing assets so they:

- clearly explain what OSM is and who it is for
- separate absorbed upstream inspiration from implemented product behavior
- state current release scope and known post-release work
- align package, desktop, and release-note versions

**Step 4: Run test to verify it passes**

Run: `rg -n "\"version\"|version =|当前已吸收|当前已实现|发布后计划" README.md docs/release/github-release-notes.md src-tauri/Cargo.toml src-tauri/tauri.conf.json web/package.json`
Expected: Public-facing copy and metadata are aligned.

**Step 5: Commit**

```bash
git add README.md docs/release/github-release-notes.md docs/release/support-matrix.md docs/release/open-source-attribution.md src-tauri/Cargo.toml src-tauri/tauri.conf.json web/package.json
git commit -m "docs: finalize release copy and version metadata"
```

### Task 3: Verify, Commit, and Publish

**Files:**
- Modify: `docs/release/github-release-notes.md`
- Publish: GitHub repository About metadata
- Publish: Git tag and GitHub release

**Step 1: Write the failing test**

Attempt a full verification and GitHub publish sequence before the final release assets are confirmed.

**Step 2: Run test to verify it fails**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: Use the result as release evidence; if it fails, publishing must stop.

**Step 3: Write minimal implementation**

After verification passes:

- commit the final release state with the configured local identity
- push the branch or mainline release state to GitHub
- set the repository description/homepage metadata
- create a tagged GitHub release using the curated notes

**Step 4: Run test to verify it passes**

Run: `git -C . status --short`
Expected: Clean worktree before or immediately after the publish commit/tag.

**Step 5: Commit**

```bash
git add -A
git commit -m "release: publish v0.2.0"
git push origin HEAD
git tag v0.2.0
git push origin v0.2.0
```
