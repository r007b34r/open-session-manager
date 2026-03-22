# Unified Session CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `SRCH-07`，让 OSM 在现有 `snapshot/doctor` 之外，新增统一的 `list/search/get/view/expand` 五类 CLI，会话查询不再只能靠整个 snapshot JSON 自己扒。

**Architecture:** 复用现有 `DashboardSnapshot` 与本地/fixture 构建链，在 Rust 侧新增一个轻量查询模块，专门负责把快照映射成 CLI 可消费的 `list/search/get/view/expand` 输出。`main.rs` 只负责命令分发与公共 flag 解析，不把搜索和格式拼装逻辑堆回入口文件。搜索先做稳定的本地 lexical 评分，覆盖 title/summary/tags/transcript/todo 五类字段；`view` 返回面向终端的 Markdown 视图；`expand` 返回机器可消费的上下文 bundle。

**Tech Stack:** Rust, serde_json, cargo test

---

### Task 1: 先写失败测试

**Files:**
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing tests**

Cover:
- `list --fixtures ...` 返回会话清单数组
- `search --fixtures ... --query relay` 返回命中、片段和匹配字段
- `get --fixtures ... --session ses-002` 返回对应会话详情
- `view --fixtures ... --session ses-002` 返回带标题/摘要/待办的 Markdown 视图
- `expand --fixtures ... --session ses-003` 返回会话、相关审计事件与 transcript/todo 扩展数据

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --test cli_snapshot list_command_emits_session_inventory`
- `cargo test --test cli_snapshot search_command_returns_ranked_hits`
- `cargo test --test cli_snapshot get_command_returns_full_session_detail`
- `cargo test --test cli_snapshot view_command_renders_markdown_summary`
- `cargo test --test cli_snapshot expand_command_returns_context_bundle`

**Step 3: Commit**

```bash
git add src-tauri/tests/cli_snapshot.rs
git commit -m "test(cli): cover unified session query commands [SRCH-07]"
```

### Task 2: 实现查询层与命令分发

**Files:**
- Create: `src-tauri/src/commands/query.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Write minimal implementation**

- 抽一个共享快照加载入口，统一处理 `--fixtures` 和 `--audit-db`
- 提供 `list/search/get/view/expand` 五类查询函数
- `search` 输出稳定命中评分、命中字段和 snippet
- `view` 输出 Markdown 文本
- `expand` 输出 `session + auditEvents + transcript/todo` bundle

**Step 2: Run tests to verify they pass**

Run:
- `cargo test --test cli_snapshot list_command_emits_session_inventory`
- `cargo test --test cli_snapshot search_command_returns_ranked_hits`
- `cargo test --test cli_snapshot get_command_returns_full_session_detail`
- `cargo test --test cli_snapshot view_command_renders_markdown_summary`
- `cargo test --test cli_snapshot expand_command_returns_context_bundle`

**Step 3: Commit**

```bash
git add src-tauri/src/commands/query.rs src-tauri/src/commands/mod.rs src-tauri/src/main.rs
git commit -m "feat(cli): add unified session query commands [SRCH-07]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `README.md`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `scripts/verify.ps1`

**Step 1: Run verification**

Run:
- `cargo test --test cli_snapshot`

**Step 2: Update docs**

- README 补 CLI 示例
- 总 spec 把 `SRCH-07` 标成完成
- 支持矩阵与发布说明同步统一 CLI 能力
- `verify.ps1` 纳入 `cli_snapshot` 集成测试

**Step 3: Commit**

```bash
git add README.md docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md scripts/verify.ps1
git commit -m "docs(cli): record unified session query commands [SRCH-07]"
```
