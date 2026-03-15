# Open Session Manager Release Branding Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将现有项目以 `open Session Manager` / `OSM` 的对外品牌完成命名收口、验证、构建与 GitHub 发布。

**Architecture:** 保留现有 Rust 核心、Web 控制台和 Tauri 桌面架构，只收敛品牌名称、包名、可执行文件名、默认数据目录名与发布文档。实现顺序严格遵循 TDD：先让前端品牌测试失败，再最小化修改 UI 文案；随后补齐桌面端、脚本与文档命名；最后执行全量验证、构建与发布。

**Tech Stack:** Rust 2024, Tauri 2, React, TypeScript, Vitest, Playwright, PowerShell, GitHub REST API

---

### Task 1: 品牌测试红灯与前端文案

**Files:**
- Modify: `web/src/app.test.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/index.html`

**Step 1: 写失败测试**

测试已将标题期望改为 `open Session Manager` / `开放会话管理器`。

**Step 2: 运行测试确认红灯**

Run: `npm test -- src/app.test.tsx`
Expected: FAIL，原因是 UI 仍显示旧品牌。

**Step 3: 写最小实现**

- 把中英文标题替换为新品牌
- 保留浏览器语言自动检测与手动切换
- 同步页面 `<title>`

**Step 4: 运行测试确认转绿**

Run: `npm test -- src/app.test.tsx`
Expected: PASS

**Step 5: 提交**

```bash
git add web/src/app.test.tsx web/src/lib/i18n.tsx web/index.html
git commit -m "feat: rebrand ui as open session manager"
```

### Task 2: Rust / Tauri / 脚本命名收口

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/desktop.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`
- Modify: `scripts/export-dashboard-snapshot.mjs`
- Modify: `Cargo.lock`

**Step 1: 写失败测试或使用现有编译失败作为红灯**

将包名和二进制名切换后，现有 CLI 测试与脚本引用会失配。

**Step 2: 运行定向验证确认红灯**

Run: `cargo test -p open-session-manager-core cli_snapshot -- --nocapture`
Expected: FAIL，原因是包名与测试引用未对齐。

**Step 3: 写最小实现**

- 把 Cargo package、winres、Tauri product、identifier、窗口标题统一到新品牌
- 将内部目录名统一成 `open-session-manager`
- 修正脚本和测试里的包名、临时目录前缀与 CLI 可执行文件引用

**Step 4: 运行定向验证确认转绿**

Run: `cargo test -p open-session-manager-core cli_snapshot -- --nocapture`
Expected: PASS

**Step 5: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/src/lib.rs src-tauri/src/desktop.rs src-tauri/src/actions/tests.rs src-tauri/tests/cli_snapshot.rs scripts/export-dashboard-snapshot.mjs Cargo.lock
git commit -m "feat: rebrand desktop runtime as open session manager"
```

### Task 3: 文档、包信息与发布资产命名

**Files:**
- Modify: `README.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/plans/2026-03-15-open-session-manager-design.md`
- Modify: `docs/plans/2026-03-15-open-session-manager.md`
- Modify: `web/package.json`
- Modify: `web/package-lock.json`

**Step 1: 写失败检查**

Run: `rg -n "Agent Session Governance|agent-session-governance|会话治理平台|agent-session-governance-core|agent-session-governance-web"`
Expected: 返回旧品牌残留。

**Step 2: 写最小实现**

- 对外名称统一为 `open Session Manager`
- 中文标题统一为 `开放会话管理器`
- 仓库名、包名、构建路径、发布文档统一更新
- 在文档中保留简称 `OSM`

**Step 3: 运行残留扫描确认转绿**

Run: `rg -n "Agent Session Governance|agent-session-governance|会话治理平台|agent-session-governance-core|agent-session-governance-web"`
Expected: 仅剩历史调研内容或确认为无需替换的上下文；发布链路相关文件无旧品牌残留。

**Step 4: 提交**

```bash
git add README.md docs/release/github-release-notes.md docs/release/support-matrix.md docs/plans/2026-03-15-open-session-manager-design.md docs/plans/2026-03-15-open-session-manager.md web/package.json web/package-lock.json
git commit -m "docs: align release materials with open session manager"
```

### Task 4: 全量验证、构建与 GitHub 发布

**Files:**
- Modify: `.git/config` or local git config as needed
- Create: `temp/github-release-*.zip`

**Step 1: 运行全量验证**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: exit 0

**Step 2: 运行 release 构建**

Run: `npm --prefix web run tauri:build`
Expected: 生成新的 release 可执行文件

**Step 3: 打包发布资产**

Run: `Compress-Archive ...`
Expected: 生成包含文档与可执行文件的 zip 包

**Step 4: 合并与发布**

- 把特性分支合并回 `main`
- 创建 GitHub 仓库 `open-session-manager`
- 推送源码、标签与发布资产

**Step 5: 最终验证**

Run: `git status --short`
Expected: 工作树干净，仅保留预期的构建产物目录之外的未跟踪文件
