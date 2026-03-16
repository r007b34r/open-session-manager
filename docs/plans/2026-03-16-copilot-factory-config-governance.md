# Copilot Factory Config Governance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `GitHub Copilot CLI` 与 `Factory Droid` 打通真实的配置发现、风险审计、快照输出与前端展示闭环，让竞品吸收在产品行为上可见。

**Architecture:** 先通过失败测试锁定新增助手的配置解析与风险识别，再补 discovery roots、fixture snapshot 和浏览器 fallback 数据。实现尽量复用现有审计风险码与 `ConfigRiskPanel`，避免无必要新增 UI 结构，只扩真实数据面。

**Tech Stack:** Rust core, Tauri commands, Vitest, cargo test, PowerShell verification script

---

### Task 1: 锁定 Rust 配置审计行为

**Files:**
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/audit/config_audit.rs`
- Create: `tests/fixtures/configs/copilot/config.json`
- Create: `tests/fixtures/configs/copilot/mcp-config.json`
- Create: `tests/fixtures/configs/factory/settings.json`
- Create: `tests/fixtures/configs/factory/settings.local.json`

**Step 1: Write the failing test**

在 `src-tauri/src/audit/tests.rs` 增加：
- `audits_copilot_config_and_detects_enterprise_or_tool_risks`
- `audits_factory_config_and_masks_credentials`

断言点：
- 能解析 provider/model/base url
- 能识别 `third_party_base_url` / `dangerous_permissions` / `missing_primary_secret`
- 凭据会被脱敏
- MCP / allowlist / auto approve 会进入配置预览

**Step 2: Run test to verify it fails**

Run:

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test audits_copilot_config_and_detects_enterprise_or_tool_risks --lib -- --exact --test-threads=1
C:\Users\Max\.cargo\bin\cargo.exe test audits_factory_config_and_masks_credentials --lib -- --exact --test-threads=1
```

Expected:
- 因为 `audit_config()` 尚不支持这两个助手而失败

**Step 3: Write minimal implementation**

在 `src-tauri/src/audit/config_audit.rs`：
- 扩 `audit_config()` match 分支
- 新增 `audit_github_copilot_cli()` 和 `audit_factory_droid()`
- 复用 `endpoint_risk_flags()`，必要时扩官方 provider / host 判定
- 复用现有风险码，避免新增翻译负担

**Step 4: Run test to verify it passes**

Run:

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test audits_copilot_config_and_detects_enterprise_or_tool_risks --lib -- --exact --test-threads=1
C:\Users\Max\.cargo\bin\cargo.exe test audits_factory_config_and_masks_credentials --lib -- --exact --test-threads=1
```

Expected:
- 两个新增测试通过

**Step 5: Commit**

```powershell
git add src-tauri/src/audit/config_audit.rs src-tauri/src/audit/tests.rs tests/fixtures/configs/copilot tests/fixtures/configs/factory
git commit -m "feat: audit copilot and factory configs"
```

### Task 2: 锁定 discovery 与 fixture snapshot

**Files:**
- Modify: `src-tauri/src/commands/discovery.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing test**

在 `src-tauri/tests/cli_snapshot.rs` 增加断言：
- `configs.len()` 从 `5` 提升到 `7`
- fixture snapshot 中出现 `github-copilot-cli` / `factory-droid`
- 对应 config 带 model / maskedSecret / risks

必要时在 `dashboard.rs` 单测中补 fixture config 断言。

**Step 2: Run test to verify it fails**

Run:

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test snapshot_command_emits_real_dashboard_json_from_fixtures --test cli_snapshot -- --exact --test-threads=1
```

Expected:
- 因 fixture config target 尚未扩展而失败

**Step 3: Write minimal implementation**

在 `src-tauri/src/commands/discovery.rs`：
- 加入用户级 `~/.copilot/config.json`
- 加入用户级 `~/.copilot/mcp-config.json`
- 加入用户级 `~/.factory/settings.json`
- 加入用户级 `~/.factory/settings.local.json`
- Windows / WSL 同步补齐

在 `src-tauri/src/commands/dashboard.rs`：
- fixture config targets 加入 copilot / factory

**Step 4: Run test to verify it passes**

Run:

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test snapshot_command_emits_real_dashboard_json_from_fixtures --test cli_snapshot -- --exact --test-threads=1
```

Expected:
- CLI snapshot fixture 测试通过

**Step 5: Commit**

```powershell
git add src-tauri/src/commands/discovery.rs src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs
git commit -m "feat: surface copilot and factory config snapshot data"
```

### Task 3: 锁定前端 fallback 与配置展示

**Files:**
- Modify: `web/src/components/config-risk-panel.test.tsx`
- Modify: `web/src/lib/api.ts`

**Step 1: Write the failing test**

扩 `config-risk-panel.test.tsx`：
- 断言能渲染 `GitHub Copilot CLI` / `Factory Droid`
- 断言风险 badge 与脱敏 secret 可见

**Step 2: Run test to verify it fails**

Run:

```powershell
npm --prefix web run test -- config-risk-panel
```

Expected:
- 因 fallback 数据未覆盖新助手而失败

**Step 3: Write minimal implementation**

在 `web/src/lib/api.ts` 的 fallback snapshot 中加入：
- `github-copilot-cli`
- `factory-droid`

保持 runtime / usage 结构不变，只扩配置样本。

**Step 4: Run test to verify it passes**

Run:

```powershell
npm --prefix web run test -- config-risk-panel
```

Expected:
- 前端配置面板测试通过

**Step 5: Commit**

```powershell
git add web/src/components/config-risk-panel.test.tsx web/src/lib/api.ts
git commit -m "feat: expose copilot and factory config risks in web fallback"
```

### Task 4: 全量验证与文档口径同步

**Files:**
- Modify: `README.md`
- Modify: `docs/research/competitor-gap.md`
- Modify: `docs/specs/upstream-absorption-master-spec.md`
- Modify: `docs/releases/latest.md`

**Step 1: Run focused verification**

Run:

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test --lib -- --test-threads=1
C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot -- --test-threads=1
npm --prefix web run test
npm --prefix web run build
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

Expected:
- 全部通过

**Step 2: Update docs**

同步更新：
- 首页支持矩阵
- 竞品吸收对比
- 当前版本说明
- 仍未完成项的诚实缺口说明

**Step 3: Commit**

```powershell
git add README.md docs/research/competitor-gap.md docs/specs/upstream-absorption-master-spec.md docs/releases/latest.md docs/plans/2026-03-16-copilot-factory-config-governance.md
git commit -m "docs: update absorption status for config governance"
```
