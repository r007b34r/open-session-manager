# Qwen CLI Config Governance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 `Qwen CLI` 的配置治理补齐到 OSM，覆盖审计、风险提示、项目级发现、可视化编辑、备份和回滚。

**Architecture:** 继续沿用现有 `config_audit + config_writeback + dashboard snapshot + web config panel` 链路，不新增独立存储。`Qwen CLI` 配置解析以 `settings.json` 为主，同时兼容同目录 `.env` 的密钥来源，并从 `modelProviders` 推断当前模型对应的 `baseUrl` 和 `envKey`。

**Tech Stack:** Rust (`serde_json`, `rusqlite`)、React/Vitest、现有 Tauri commands/actions。

---

### Task 1: 补齐 fixtures 与失败测试

**Files:**
- Create: `tests/fixtures/configs/qwen/settings.json`
- Create: `tests/fixtures/configs/qwen/.env`
- Create: `tests/fixtures/configs/qwen/project/.qwen/settings.json`
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `web/src/components/config-risk-panel.test.tsx`

**Step 1: Write the failing test**

- 审计测试断言 `qwen-cli` 能读出 `provider/model/baseUrl`、`mcp` 与 masked secret。
- 写回测试断言 `qwen-cli` 可更新 model/baseUrl/secret，并能从备份 manifest 回滚。
- dashboard 测试断言 fixture snapshot 与 project-level discovery 会包含 `qwen-cli` config。
- Web 测试断言 `Qwen CLI` config 在 UI 中出现可编辑入口。

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml audit::tests::audits_qwen_config_and_masks_credentials`
Expected: FAIL with `unsupported assistant: qwen-cli`

**Step 3: Write minimal implementation**

- 在 `config_audit.rs` 增加 `qwen-cli` 审计。
- 在 `config_writeback.rs` 增加 `qwen-cli` 写回。
- 在 dashboard/config fixtures 接线上补齐 `qwen-cli`。
- 在前端把 `Qwen CLI` 加入可编辑助手。

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib`
Expected: PASS for新增 `qwen-cli` 用例

**Step 5: Commit**

```bash
git add docs/plans/2026-03-24-qwen-config-governance.md tests/fixtures/configs/qwen src-tauri/src/audit/tests.rs src-tauri/src/actions/tests.rs src-tauri/src/commands/dashboard.rs web/src/components/config-risk-panel.test.tsx
git commit -m "feat(config): add qwen cli governance [ADP-08]"
```

### Task 2: 统一验证与文档

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `tests/fixtures/dashboard-snapshot.golden.json`

**Step 1: Write the failing test**

- 让 snapshot golden 与 docs 基于真实新增的 `qwen-cli` config 能力更新。

**Step 2: Run test to verify it fails**

Run: `node scripts/check-fixture-snapshot.mjs`
Expected: FAIL before golden/doc updates

**Step 3: Write minimal implementation**

- 更新 support matrix / release notes / spec 状态。
- 更新 dashboard snapshot golden。

**Step 4: Run test to verify it passes**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/specs/2026-03-22-autonomous-git-tdd-program.md tests/fixtures/dashboard-snapshot.golden.json
git commit -m "docs(release): document qwen config governance [ADP-08]"
```
