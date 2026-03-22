# Search API Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `SRCH-08`，让现有 Tauri 查询 API 不只是“能查”，而是支持分页、助手过滤和排序，满足统一 search API 的最低可用面。

**Architecture:** 继续复用 `commands::query` 作为共享查询层，不再在 `desktop.rs` 里手写过滤。新增可序列化请求结构，把 `assistant / limit / offset / sortBy / descending` 传进 list/search API。CLI 维持当前简单形态，Tauri API 先拿到稳定的分页、过滤、排序能力。

**Tech Stack:** Rust, Tauri command, cargo test

---

### Task 1: 先写失败测试

**Files:**
- Modify: `src-tauri/src/desktop.rs`

**Step 1: Write the failing tests**

Cover:
- `list_session_inventory` 支持 `assistant` 过滤、`limit/offset` 分页和 `lastActivityAt` 排序
- `search_session_inventory` 支持 `assistant` 过滤、`limit/offset` 分页和 `title` 排序

**Step 2: Run tests to verify they fail**

Run:
- `cargo test desktop_query_api_supports_pagination_filtering_and_sorting --manifest-path src-tauri/Cargo.toml`

**Step 3: Commit**

```bash
git add src-tauri/src/desktop.rs
git commit -m "test(api): cover search api pagination and sorting [SRCH-08]"
```

### Task 2: 实现请求参数和共享查询排序层

**Files:**
- Modify: `src-tauri/src/commands/query.rs`
- Modify: `src-tauri/src/desktop.rs`

**Step 1: Write minimal implementation**

- 新增 list/search 请求结构
- 共享查询层支持 assistant filter、offset/limit、sortable fields
- Tauri command 接收请求并返回稳定 JSON

**Step 2: Run tests to verify they pass**

Run:
- `cargo test desktop_query_api_supports_pagination_filtering_and_sorting --manifest-path src-tauri/Cargo.toml`

**Step 3: Commit**

```bash
git add src-tauri/src/commands/query.rs src-tauri/src/desktop.rs
git commit -m "feat(api): add pagination and sorting to search api [SRCH-08]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `cargo test desktop_query_commands_surface_shared_session_payloads --manifest-path src-tauri/Cargo.toml`
- `cargo test desktop_query_api_supports_pagination_filtering_and_sorting --manifest-path src-tauri/Cargo.toml`

**Step 2: Update docs**

- 总 spec 把 `SRCH-08` 标成完成
- README 与发布文档补 assistant filter / pagination / sorting 能力

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md README.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(api): record search api filtering and sorting [SRCH-08]"
```
