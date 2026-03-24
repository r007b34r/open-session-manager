# Git File Preview Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成 `GIT-07`，让 Git 文件树支持点击文件后加载只读预览，并在面板内展示路径、大小和文本内容。

**Architecture:** 复用 `GIT-06` 的文件树条目，但文件内容改为按需加载。Tauri 侧新增安全只读命令，严格限制路径逃逸和预览大小；Web demo 模式使用本地 mock 预览数据兜底，避免浏览器模式失去交互。

**Tech Stack:** Rust `std::fs`、Tauri invoke、React、Vitest、Rust unit tests

---

### Task 1: 建立只读预览契约

**Files:**
- Modify: `src-tauri/src/desktop.rs`
- Modify: `web/src/lib/api.ts`

**Step 1: Write the failing test**

- 在 Rust desktop 测试中断言：
  - 可以读取 repo 内文本文件预览
  - `..` 路径逃逸会被拒绝

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml desktop::tests::git_project_file_preview_reads_files_and_rejects_escape --lib -- --exact`

**Step 3: Write minimal implementation**

- 新增 `preview_git_project_file` Tauri 命令
- 限制预览大小，按文本读取 repo 内文件
- 统一返回 `repoRoot / relativePath / content / truncated / byteSize / lineCount`

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml desktop::tests::git_project_file_preview_reads_files_and_rejects_escape --lib -- --exact`

### Task 2: 渲染只读 editor

**Files:**
- Modify: `web/src/components/git-project-panel.tsx`
- Modify: `web/src/components/git-project-panel.test.tsx`
- Modify: `web/src/app.tsx`
- Modify: `web/src/routes/index.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write the failing test**

- 在 `git-project-panel.test.tsx` 或 `app.test.tsx` 断言：
  - 点击文件树中的文件会显示只读 preview/editor
  - 会展示相对路径和大小
  - 目录条目不可触发文件预览

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- src/components/git-project-panel.test.tsx`

**Step 3: Write minimal implementation**

- Git 面板维护当前选中文件和加载态
- 通过 props 回调或 API helper 拉取预览
- 用 `textarea[readOnly]` 或 `pre` 展示只读内容

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- src/components/git-project-panel.test.tsx`

### Task 3: 全量验证与 spec 收口

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Focused verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml desktop::tests::git_project_file_preview_reads_files_and_rejects_escape --lib -- --exact
npm --prefix web run test -- src/components/git-project-panel.test.tsx
```

**Step 2: Full verification**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 3: Update spec**

- 把 `GIT-07` 从 `todo` 改成 `done`
- 写明“Git 文件树支持按需只读预览与 editor 样式查看，并通过路径逃逸保护测试”
