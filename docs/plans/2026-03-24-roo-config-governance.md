# Roo Code Config Governance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Status:** Completed on 2026-03-24

**Goal:** 把 `Roo Code` 的配置治理补齐到 OSM，覆盖全局 provider profile 审计、全局/项目级 MCP 配置发现、可视化编辑、备份和回滚。

**Architecture:** 继续沿用现有 `config_audit + config_writeback + dashboard snapshot + web config panel` 链路，不新增独立存储。`Roo Code` 的主配置读取 `.../globalStorage/rooveterinaryinc.roo-cline/settings/roo-code-settings.json`，并兼容同目录 `mcp_settings.json` / `cline_mcp_settings.json`；项目级配置通过会话 `projectPath` 派生 `.roo/mcp.json`。主配置优先读取 `providerProfiles.currentApiConfigName` 指向的 profile，并解析 `apiProvider/openAiBaseUrl/openAiModelId/openAiApiKey` 一类字段。

**Tech Stack:** Rust (`serde_json`, `rusqlite`)、React/Vitest、现有 Tauri commands/actions、fixture-driven TDD。

---

### Task 1: 补齐 fixtures 与失败测试

**Files:**
- Create: `tests/fixtures/configs/roo/roo-code-settings.json`
- Create: `tests/fixtures/configs/roo/mcp_settings.json`
- Create: `tests/fixtures/configs/roo/project/.roo/mcp.json`
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `web/src/components/config-risk-panel.test.tsx`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing test**

- 审计测试断言 `roo-code` 能读出当前 profile 的 `provider/model/baseUrl`、全局 MCP server、风险标记和 masked secret。
- dashboard 测试断言 fixture snapshot 与本地 project-level discovery 会包含 `roo-code` 全局配置和项目级 `.roo/mcp.json`。
- 写回测试断言 `roo-code` 可更新当前 profile 的 `model/baseUrl/secret`，并能从备份 manifest 回滚。
- Web 测试断言 `Roo Code` config 会出现在配置面板并支持编辑入口。
- CLI snapshot 测试断言 fixture snapshot 会带出 `roo-code` 配置记录。

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml audit::tests::audits_roo_code_config_and_masks_credentials`

Expected: FAIL with `unsupported assistant: roo-code`

**Step 3: Write minimal implementation**

- 只写最小 fixture 和断言，不提前写实现。

**Step 4: Run test to verify it fails for the right reason**

Run: `cargo test --manifest-path src-tauri/Cargo.toml audit::tests::audits_roo_code_config_and_masks_credentials`

Expected: FAIL，且失败原因仍是 `unsupported assistant: roo-code`

### Task 2: 接入 Roo Code 配置审计与项目级发现

**Files:**
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/audit/config_audit.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write minimal implementation**

- 在 `discover_known_roots()` 里新增 `roo-code` 全局配置路径：
  - Windows: `%APPDATA%\\Code\\User\\globalStorage\\rooveterinaryinc.roo-cline\\settings\\roo-code-settings.json`
  - WSL/Linux: `~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/roo-code-settings.json`
- 在 `audit_config()` 增加 `roo-code` 分支。
- `audit_roo_code()`：
  - 读取 `roo-code-settings.json`
  - 定位 `providerProfiles.currentApiConfigName`
  - 从当前 profile 提取 `apiProvider/openAiBaseUrl/openAiModelId/openAiApiKey`
  - 读取同目录 `mcp_settings.json`，若不存在则回退 `cline_mcp_settings.json`
  - 生成 MCP records、masked secret、endpoint 风险
- 在 `derive_project_config_targets()` 里为 `roo-code` 派生 `<project>/.roo/mcp.json`
- 对项目级 `.roo/mcp.json`，复用 `roo-code` 审计分支读取 MCP 列表，并输出只读 config record

**Step 2: Run tests to verify they pass**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml audits_roo_code_config_and_masks_credentials
cargo test --manifest-path src-tauri/Cargo.toml local_snapshot_discovers_roo_code_user_and_project_configs
```

Expected: PASS

### Task 3: 接入 Roo Code 安全写回与前端编辑

**Files:**
- Modify: `src-tauri/src/actions/config_writeback.rs`
- Modify: `web/src/components/config-risk-panel.tsx`
- Modify: `web/src/lib/provider-presets.ts`

**Step 1: Write minimal implementation**

- `config_writeback` 增加 `roo-code`
- 只允许编辑当前 profile 的 `model/baseUrl/secret`
- 不允许 provider rename，避免 profile 结构漂移
- 备份 manifest 同时纳入 `roo-code-settings.json` 与同目录 `mcp_settings.json` / `cline_mcp_settings.json`（若存在）
- Web 面板把 `Roo Code` 加入可编辑助手列表
- 如无稳定 preset catalog，就先不暴露 Roo 专属 preset，避免伪能力

**Step 2: Run tests to verify they pass**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml writes_back_and_rolls_back_roo_code_config_with_backup_and_audit
npm --prefix web run test -- src/components/config-risk-panel.test.tsx
```

Expected: PASS

### Task 4: 收口 snapshot、文档与验证

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `tests/fixtures/dashboard-snapshot.golden.json`
- Modify: `tests/fixtures/fixture-ledger.json`

**Step 1: Update docs and golden**

- README / 支持矩阵 / release notes 补 `Roo Code` 配置治理
- 总 spec 把 `ADP-09` 从 `partial` 改成 `done`
- 重新生成 fixture ledger 和 snapshot golden

**Step 2: Run full verification**

Run:

```powershell
node scripts/fixture-ledger.mjs --write
node scripts/check-fixture-snapshot.mjs --write
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

Expected: PASS

### Task 5: 记录 Git TDD 证据并提交

**Files:**
- Modify: `docs/plans/2026-03-24-roo-config-governance.md`

**Step 1: Record checkpoints**

Run:

```powershell
node scripts/git-review-snapshot.mjs --item ADP-09 --phase verify --note "Roo Code config governance passed full verify"
node scripts/git-tdd-checkpoint.mjs --item ADP-09 --phase verify --note "Roo Code config governance passed full verify"
```

**Step 2: Commit**

Run:

```powershell
git add docs/plans/2026-03-24-roo-config-governance.md README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/specs/2026-03-22-autonomous-git-tdd-program.md src-tauri/src/audit/config_audit.rs src-tauri/src/actions/config_writeback.rs src-tauri/src/commands/discovery.rs src-tauri/src/commands/dashboard.rs src-tauri/src/audit/tests.rs src-tauri/src/actions/tests.rs src-tauri/tests/cli_snapshot.rs web/src/components/config-risk-panel.tsx web/src/components/config-risk-panel.test.tsx web/src/lib/provider-presets.ts tests/fixtures
git commit -m "feat(config): add roo code governance [ADP-09]"
```
