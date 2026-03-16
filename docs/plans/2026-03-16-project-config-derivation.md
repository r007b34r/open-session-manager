# Project Config Derivation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 OSM 能根据已发现会话的 `projectPath` 自动发现 `GitHub Copilot CLI` 与 `Factory Droid` 的项目级配置，并在快照与配置面板中展示出来。

**Architecture:** 保持当前 discovery 层只负责用户级 roots，把项目级配置发现放在 `build_snapshot()` 中，通过已解析出的 session `projectPath` 派生额外 `ConfigAuditTarget`。这样不需要全盘扫盘，也能把配置精确绑定到真实会话项目。Copilot 项目级配置支持 `settings.json + settings.local.json` 合并，Factory 继续复用 `settings.json + settings.local.json` 覆盖模型。

**Tech Stack:** Rust core, Tauri snapshot pipeline, cargo test

---

### Task 1: 锁定项目级配置解析行为

**Files:**
- Modify: `src-tauri/src/audit/tests.rs`
- Modify: `src-tauri/src/audit/config_audit.rs`
- Create: `tests/fixtures/configs/copilot/project/.github/copilot/settings.json`
- Create: `tests/fixtures/configs/copilot/project/.github/copilot/settings.local.json`
- Create: `tests/fixtures/configs/factory/project/.factory/settings.json`
- Create: `tests/fixtures/configs/factory/project/.factory/settings.local.json`

**Step 1: Write the failing test**

新增审计测试：
- `audits_copilot_project_settings_and_prefers_local_override`
- `audits_factory_project_settings_local_scope`

断言：
- Copilot 项目级配置会合并 `settings.json` 与 `settings.local.json`
- Factory 项目级配置会优先展示本地覆盖层
- `scope = project`
- model / endpoint / 风险标记正确

**Step 2: Run test to verify it fails**

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test audits_copilot_project_settings_and_prefers_local_override --lib -- --test-threads=1
C:\Users\Max\.cargo\bin\cargo.exe test audits_factory_project_settings_local_scope --lib -- --test-threads=1
```

Expected:
- 当前 Copilot 审计不支持 `settings.json/settings.local.json` 组合而失败

**Step 3: Write minimal implementation**

在 `src-tauri/src/audit/config_audit.rs`：
- 扩 `audit_github_copilot_cli()` 让它支持项目级 `settings.json + settings.local.json`
- 保持用户级 `config.json + mcp-config.json` 兼容
- Factory 沿用现有 merge 行为，只补测试覆盖

**Step 4: Run test to verify it passes**

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test audits_copilot_project_settings_and_prefers_local_override --lib -- --test-threads=1
C:\Users\Max\.cargo\bin\cargo.exe test audits_factory_project_settings_local_scope --lib -- --test-threads=1
```

**Step 5: Commit**

```powershell
git add src-tauri/src/audit/config_audit.rs src-tauri/src/audit/tests.rs tests/fixtures/configs/copilot/project tests/fixtures/configs/factory/project
git commit -m "feat: support project config parsing for copilot and factory"
```

### Task 2: 锁定快照中的项目级配置发现

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing test**

新增快照测试：
- `local_snapshot_discovers_project_level_copilot_and_factory_configs`

构造：
- 临时 home 下放 Copilot / Factory 会话文件
- 会话中的 `projectPath/cwd` 指向临时项目目录
- 项目目录里放对应项目级配置

断言：
- `snapshot.configs` 出现 `scope = project`
- Copilot 指向 `.github/copilot/settings.local.json`
- Factory 指向 `.factory/settings.local.json`

**Step 2: Run test to verify it fails**

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test local_snapshot_discovers_project_level_copilot_and_factory_configs --lib -- --test-threads=1
```

Expected:
- 当前 `build_snapshot()` 还没有从 session 项目路径派生 config target 而失败

**Step 3: Write minimal implementation**

在 `src-tauri/src/commands/dashboard.rs`：
- `build_snapshot()` 先拿到 sessions
- 根据 sessions 派生项目级 config targets：
  - Copilot: `<project>/.github/copilot/settings.json`
  - Factory: `<project>/.factory/settings.local.json`
- 与用户级 targets 合并并去重

**Step 4: Run test to verify it passes**

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test local_snapshot_discovers_project_level_copilot_and_factory_configs --lib -- --test-threads=1
```

**Step 5: Commit**

```powershell
git add src-tauri/src/commands/dashboard.rs
git commit -m "feat: derive project config targets from indexed sessions"
```

### Task 3: 回归验证与口径同步

**Files:**
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run full verification**

```powershell
C:\Users\Max\.cargo\bin\cargo.exe test --lib -- --test-threads=1
C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot -- --test-threads=1
npm --prefix web run test
npm --prefix web run build
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

**Step 2: Update release docs**

同步把口径从“用户级配置治理”推进到“支持按会话派生项目级配置发现”。

**Step 3: Commit**

```powershell
git add README.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/plans/2026-03-16-project-config-derivation.md
git commit -m "docs: note project config discovery coverage"
```
