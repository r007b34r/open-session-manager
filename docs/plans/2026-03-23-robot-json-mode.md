# Robot JSON Mode Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `API-05`，让 OSM CLI 提供显式的机器模式 JSON 输出，同时继续复用现有 HTTP JSON 契约。

**Architecture:** 不新增新命令，只在现有 `snapshot/doctor/list/search/get/view/expand` CLI 上增加全局 `--json` 开关。默认保持当前 pretty JSON；显式 `--json` 时输出紧凑稳定 JSON，便于 shell 脚本、自动化工具和后续 remote shell 复用。

**Tech Stack:** Rust, cargo test

---

### Task 1: 先写机器模式红测

**Files:**
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing tests**

Cover:
- `list --json` 输出可解析的紧凑 JSON
- `view --json` 仍输出稳定 JSON envelope，而不是裸 Markdown

**Step 2: Run tests to verify they fail**

Run:
- `cargo test robot_json_mode_emits_compact_cli_payloads --test cli_snapshot --manifest-path src-tauri/Cargo.toml`

**Step 3: Commit**

```bash
git add src-tauri/tests/cli_snapshot.rs
git commit -m "test(api): cover robot json mode for cli [API-05]"
```

### Task 2: 实现 `--json` 机器模式

**Files:**
- Modify: `src-tauri/src/main.rs`

**Step 1: Write minimal implementation**

- 增加全局 `--json` 检测
- `--json` 时使用紧凑 JSON 输出
- 保持现有默认 pretty JSON 行为和现有命令语义

**Step 2: Run tests to verify they pass**

Run:
- `cargo test robot_json_mode_emits_compact_cli_payloads --test cli_snapshot --manifest-path src-tauri/Cargo.toml`

**Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(api): add robot json mode to cli [API-05]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Run verification**

Run:
- `cargo test --test cli_snapshot --manifest-path src-tauri/Cargo.toml`
- `cargo test --test http_api --manifest-path src-tauri/Cargo.toml`

**Step 2: Update docs**

- README 补 `--json` 用法
- 支持矩阵和 release notes 记录 CLI 机器模式
- 总 spec 把 `API-05` 标成 `done`

**Step 3: Commit**

```bash
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs(api): record robot json mode [API-05]"
```
