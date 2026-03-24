# Qwen CLI And Roo Code Adapters Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 `Qwen CLI` 和 `Roo Code` 的本地会话日志真正接入 OSM，让它们能出现在发现、会话详情、摘要、usage 和搜索基线里，完成 `ADP-08 / ADP-09` 的首轮可用落地。

**Architecture:** 继续沿用现有 Rust adapter + dashboard snapshot 架构，不引入额外存储层。`Qwen CLI` 采用 JSONL 聚合解析，兼容 `sessionId/session_id`、`usageMetadata` 与常见消息文本字段；`Roo Code` 以 `ui_messages.json` 为主日志，结合同目录 `api_conversation_history.json` 补模型与 agent 元数据。两者都要接入 discovery roots、narrative、transcript 和 usage 提取，避免只做到“能发现文件”。

**Tech Stack:** Rust, cargo test, fixture-driven TDD, JSON/JSONL parsing, fixture ledger and golden snapshot tooling

---

### Task 1: 先写失败测试和 fixture

**Files:**
- Create: `tests/fixtures/qwen/projects/demo-project/chats/session-alpha.jsonl`
- Create: `tests/fixtures/roocode/tasks/task-review/ui_messages.json`
- Create: `tests/fixtures/roocode/tasks/task-review/api_conversation_history.json`
- Modify: `src-tauri/src/adapters/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/usage.rs`
- Modify: `src-tauri/src/commands/discovery.rs`

**Step 1: Write the failing tests**

- 新增 `qwen_adapter_discovers_and_parses_fixture`
- 新增 `roocode_adapter_discovers_and_parses_fixture`
- 新增 discovery roots 断言，确认 `~/.qwen/projects` 与 `~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/tasks` 被发现
- 在 dashboard / usage 测试里断言 Qwen 和 Roo 会话进入 snapshot，并带 usage / transcript

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml qwen_adapter_discovers_and_parses_fixture -- --exact
cargo test --manifest-path src-tauri/Cargo.toml roocode_adapter_discovers_and_parses_fixture -- --exact
```

Expected: FAIL，原因为适配器模块和 roots 尚未实现。

### Task 2: 实现最小 adapter 和 roots

**Files:**
- Create: `src-tauri/src/adapters/qwen_cli.rs`
- Create: `src-tauri/src/adapters/roo_code.rs`
- Modify: `src-tauri/src/adapters/mod.rs`
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write minimal implementation**

- `QwenCliAdapter` 识别 `.../.qwen/projects/*/chats/*.jsonl`
- `RooCodeAdapter` 识别 `.../rooveterinaryinc.roo-cline/tasks/*/ui_messages.json`
- dashboard 的 `session_adapter`、fixture roots 和未知文件诊断逻辑接入两类助手

**Step 2: Run tests to verify they pass**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml qwen_adapter_discovers_and_parses_fixture -- --exact
cargo test --manifest-path src-tauri/Cargo.toml roocode_adapter_discovers_and_parses_fixture -- --exact
```

Expected: PASS。

### Task 3: 补 narrative / transcript / usage 与基线

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/transcript/mod.rs`
- Modify: `src-tauri/src/usage.rs`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `tests/fixtures/fixture-ledger.json`
- Modify: `tests/fixtures/dashboard-snapshot.golden.json`

**Step 1: Write minimal implementation**

- `Qwen` 支持主题、摘要、usageMetadata tokens、transcript highlights
- `Roo Code` 支持 `api_req_started` usage、UI highlights、历史环境块中的 model/agent 元数据
- 主 spec 将 `ADP-08 / ADP-09` 标记为 `done`

**Step 2: Run focused verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
node scripts/fixture-ledger.mjs --write
node scripts/check-fixture-snapshot.mjs --write
```

Expected: Rust 测试通过，fixture ledger 和 snapshot golden 完成更新。

### Task 4: 记录 Git TDD 证据并提交

**Files:**
- Modify: `docs/plans/2026-03-24-qwen-roocode-adapters.md`

**Step 1: Record checkpoints**

Run:

```powershell
node scripts/git-review-snapshot.mjs --item ADP-08 --phase green --command "cargo test --manifest-path src-tauri/Cargo.toml qwen_adapter_discovers_and_parses_fixture -- --exact"
node scripts/git-tdd-checkpoint.mjs --item ADP-08 --phase green --note "qwen adapter and snapshot integration turned green"
node scripts/git-review-snapshot.mjs --item ADP-09 --phase green --command "cargo test --manifest-path src-tauri/Cargo.toml roocode_adapter_discovers_and_parses_fixture -- --exact"
node scripts/git-tdd-checkpoint.mjs --item ADP-09 --phase green --note "roo code adapter and snapshot integration turned green"
node scripts/git-review-snapshot.mjs --item ADP-09 --phase verify --command "powershell -ExecutionPolicy Bypass -File scripts/verify.ps1"
node scripts/git-tdd-checkpoint.mjs --item ADP-08 --phase verify --note "qwen fixtures and snapshot are stable"
node scripts/git-tdd-checkpoint.mjs --item ADP-09 --phase verify --note "roo code fixtures and snapshot are stable"
```

**Step 2: Commit**

Run:

```powershell
git add docs/plans/2026-03-24-qwen-roocode-adapters.md src-tauri/src/adapters src-tauri/src/commands/discovery.rs src-tauri/src/commands/dashboard.rs src-tauri/src/transcript/mod.rs src-tauri/src/usage.rs tests/fixtures docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "feat(adapters): add qwen and roo code sessions [ADP-08][ADP-09]"
```
