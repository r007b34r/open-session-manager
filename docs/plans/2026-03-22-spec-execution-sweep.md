# OSM Spec Execution Sweep Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 按现有全部 spec 的优先级持续执行未完成项，先收敛浏览器运行时真实性与交互正确性，再推进 P0 发布阻塞项。

**Architecture:** 先用浏览器运行时修复专项清掉当前用户已复现的真值和交互错误，再回到总 backlog，按 `autonomous-git-tdd-program` 的 P0 顺序推进。每个 item 都必须有失败测试、最小实现、验证命令和 Git 检查点，避免再次出现“描述已吸收但功能未落地”。

**Tech Stack:** Rust, Tauri, React, TypeScript, Vitest, Playwright, PowerShell, Git notes/tags

---

### Task 1: FIX-02 会话选中与滚动解耦收口

**Files:**
- Modify: `web/src/app.tsx`
- Test: `web/src/app.test.tsx`
- Test: `tests/e2e/open-session-manager.spec.ts`

**Step 1: 运行定向测试确认当前状态**

Run: `npm --prefix web run test -- src/app.test.tsx`
Expected: 与首页嵌入式会话选择相关的测试结果可复现。

**Step 2: 运行滚动相关 E2E**

Run: `npm --prefix web run e2e -- --grep "keeps the overview page scroll position stable when selecting an embedded session"`
Expected: 记录 `scrollY` 与路由行为，不允许异常跳变。

**Step 3: 用最小实现保证选中状态与 hash 滚动解耦**

Implement in `web/src/app.tsx`:
- 首页嵌入区点击只更新本地 `selectedSessionId`
- `/sessions` 路由内使用受控历史更新
- 刷新时仍可从 hash 恢复目标会话

**Step 4: 重新运行单测与 E2E**

Run: `npm --prefix web run test -- src/app.test.tsx`
Run: `npm --prefix web run e2e -- --grep "keeps the overview page scroll position stable when selecting an embedded session"`
Expected: 全部通过。

**Step 5: 留下 Git 验证证据**

Run: `node scripts/git-tdd-checkpoint.mjs --item FIX-02 --phase verify --note "会话选中与页面滚动已解耦"`

### Task 2: FIX-03 成本语义修复

**Files:**
- Modify: `src-tauri/src/usage.rs`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/components/usage-panel.tsx`
- Test: `src-tauri/src/usage.rs`
- Test: `web/src/lib/api.test.ts`
- Test: `web/src/components/session-detail.test.tsx`
- Test: `web/src/components/usage-panel.test.tsx`

**Step 1: 写失败测试**

Cover:
- 无可靠成本依据时显示“未知”而不是 `$0.00`
- 真实零成本仍显示 `$0.00`
- 总览和详情保持一致

**Step 2: 运行相关测试确认失败**

Run: `cargo test usage`
Run: `npm --prefix web run test -- src/lib/api.test.ts src/components/session-detail.test.tsx src/components/usage-panel.test.tsx`
Expected: 至少一条测试因当前把未知成本折叠为 `0` 而失败。

**Step 3: 实现最小语义修复**

Implement:
- Rust 侧区分 `Some(0.0)` 与 `None`
- 前端 snapshot 归一化保持未知语义
- 组件层改为按语义渲染未知成本文案

**Step 4: 重新运行相关测试**

Run: `cargo test usage`
Run: `npm --prefix web run test -- src/lib/api.test.ts src/components/session-detail.test.tsx src/components/usage-panel.test.tsx`
Expected: 全部通过。

**Step 5: 运行一轮 Web 回归**

Run: `npm --prefix web run test`
Run: `npm --prefix web run e2e`
Expected: 浏览器运行时修复专项仍然稳定。

### Task 3: 回到 P0 backlog 执行顺序

**Files:**
- Reference: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Reference: `docs/specs/2026-03-16-upstream-absorption-master-spec.md`

**Step 1: 逐项确认 P0 item 的当前状态**

Track:
- `UX-07`
- `CFG-01` ~ `CFG-05`
- `MCP-09`
- `SRCH-01` / `SRCH-02` / `SRCH-09` / `SRCH-10`
- `SES-02` / `SES-03` / `SES-07`
- `WRK-01`
- `API-08`
- `QLT-01` ~ `QLT-03`

**Step 2: 每个 item 先补 plan/red 证据**

Run for each item:
- `node scripts/git-review-snapshot.mjs --item <ID> --phase plan --command "<test command>"`
- `node scripts/git-tdd-checkpoint.mjs --item <ID> --phase plan --note "<scope>"`

**Step 3: 按 item 执行 Red-Green-Verify**

Requirement:
- 先失败测试
- 再最小实现
- 再相关全量验证
- commit message 必须带 item ID

**Step 4: 更新文档与发布口径**

Update after each meaningful tranche:
- `README.md`
- 支持矩阵
- 竞品吸收 ledger
- 发布说明

### Task 4: 最终验证与本地审查

**Files:**
- Modify: `scripts/verify.ps1`
- Reference: `.git/osm/reviews/`

**Step 1: 跑统一验证**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: 本轮相关验证全部通过。

**Step 2: 保留 review 快照**

Run: `node scripts/git-review-snapshot.mjs --item RELEASE --phase review --command "powershell -ExecutionPolicy Bypass -File scripts/verify.ps1"`

**Step 3: 本地提交**

Run:
- `git add <files>`
- `git commit -m "fix(web): complete browser runtime remediation tranche [FIX-02][FIX-03]"`

**Step 4: 继续下一条未完成 spec**

Stop condition:
- 只有当前 tranche 的测试、文档和 Git 证据链都落完，才允许切换到下一个 backlog item。
