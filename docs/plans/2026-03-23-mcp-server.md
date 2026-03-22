# MCP Server Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `MCP-01`，让 OSM 通过本地 `stdio` MCP server 暴露 `list/search/get` 三类会话查询能力。

**Architecture:** 新增独立 `mcp_server` 模块，通过换行分隔 JSON-RPC 处理 `initialize`、`notifications/initialized`、`tools/list` 和 `tools/call`。工具内部继续复用现有 `commands::query` 和 snapshot 加载逻辑，不碰桌面控制链路。

**Tech Stack:** Rust, serde_json, cargo test

---

### Task 1: 先写 MCP 红测

**Files:**
- Create: `src-tauri/tests/mcp_server.rs`

**Step 1: Write the failing tests**

Cover:
- `initialize` 返回 server info 和 `tools` capability
- `tools/list` 返回 `list_sessions/search_sessions/get_session`
- `tools/call` 能调用上述三类工具并返回 JSON 文本结果

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --test mcp_server --manifest-path src-tauri/Cargo.toml`

**Step 3: Commit**

```bash
git add src-tauri/tests/mcp_server.rs
git commit -m "test(mcp): cover session query mcp server [MCP-01]"
```

### Task 2: 实现最小 MCP server

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Create: `src-tauri/src/mcp_server.rs`

**Step 1: Write minimal implementation**

- 增加 `mcp` CLI 子命令
- 通过 `stdio` 读取换行分隔 JSON-RPC
- 支持 `initialize`、`tools/list`、`tools/call`
- 把 `list/search/get` 复用为 MCP tools

**Step 2: Run tests to verify they pass**

Run:
- `cargo test --test mcp_server --manifest-path src-tauri/Cargo.toml`

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main.rs src-tauri/src/mcp_server.rs
git commit -m "feat(mcp): add session query mcp server [MCP-01]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Run verification**

Run:
- `cargo test --test mcp_server --manifest-path src-tauri/Cargo.toml`
- `cargo test --test cli_snapshot --manifest-path src-tauri/Cargo.toml`

**Step 2: Update docs**

- README 补 `mcp` 启动方式
- 支持矩阵和 release notes 记录 MCP `list/search/get`
- 总 spec 把 `MCP-01` 标成 `done`

**Step 3: Commit**

```bash
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs(mcp): record session query mcp server [MCP-01]"
```
