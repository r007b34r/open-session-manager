# 浏览器运行时修复专项 spec

日期：2026-03-22  
分支：`feat/usability-clarity`  
负责人：`r007b34r`

---

## 1. 问题背景

当前浏览器版已经被用户实测出三类明显问题：

1. 本机并不存在的 `GitHub Copilot CLI`、`Factory Droid` 配置被展示出来。
2. 在 Sessions 页面点击会话后，页面会自动滚动到异常位置。
3. 成本字段显示 `$0.00`，但没有区分“真实 0 成本”和“没有可靠成本依据”。

这不是三个孤立 UI bug，而是三条逻辑链都缺少“真值边界”。

## 2. 逻辑流审计

### 2.1 浏览器数据流

浏览器版当前的数据读取顺序在 [api.ts](C:/Users/Max/Desktop/2026年3月15日/.worktrees/feat-usability-clarity/web/src/lib/api.ts) 里是：

1. 尝试 Tauri `load_dashboard_snapshot`
2. 失败后请求 `/dashboard-snapshot.json`
3. 再失败就直接退回内置 `fallbackSnapshot`

`fallbackSnapshot` 不是简单示例会话，而是一整套虚构运行时：

- 示例 sessions
- 示例 configs
- 示例 usageOverview
- 示例 auditEvents
- 示例 runtime 路径

因此浏览器模式如果没有预先导出真实快照文件，就会整包展示一套假数据。用户看到的 `~/.copilot/config.json`、`https://copilot.enterprise-relay.example`、`***7890` 就来自这里。

### 2.2 会话选中流

选中会话时，[app.tsx](C:/Users/Max/Desktop/2026年3月15日/.worktrees/feat-usability-clarity/web/src/app.tsx) 会直接写：

`window.location.hash = "/sessions/<sessionId>"`

这意味着“选中状态”和“浏览器锚点”被绑在一起。浏览器对 hash 变更自带滚动语义，所以会话选中被错误放大成页面跳动。

### 2.3 成本显示流

成本提取和渲染目前经过：

- [usage.rs](C:/Users/Max/Desktop/2026年3月15日/.worktrees/feat-usability-clarity/src-tauri/src/usage.rs)
- [api.ts](C:/Users/Max/Desktop/2026年3月15日/.worktrees/feat-usability-clarity/web/src/lib/api.ts)
- [usage-panel.tsx](C:/Users/Max/Desktop/2026年3月15日/.worktrees/feat-usability-clarity/web/src/components/usage-panel.tsx)
- [session-detail.tsx](C:/Users/Max/Desktop/2026年3月15日/.worktrees/feat-usability-clarity/web/src/components/session-detail.tsx)

现在 `costUsd` 被建模成纯数字。没有可靠成本时也会落成 `0`，渲染层又直接格式化成 `$0.00`。这把三种情况混成了一种：

- 真正零成本
- 无成本字段
- 无法可靠提取成本

## 3. 修复顺序

### FIX-01 浏览器模式真值修复

目标：

- 浏览器模式默认不能展示任何内置样例配置、路径、usage、审计记录
- demo 数据只能显式开启，不能默默回退

验收：

- 没有 Tauri、没有 `/dashboard-snapshot.json` 时：
  - `sessions` 为空
  - `configs` 为空
  - `auditEvents` 为空
  - `usageOverview` 不是内置样例
  - `runtime` 不再展示虚构 `C:/Users/Max/...` 路径
- 只有显式开启 demo 模式时，内置样例才允许出现

### FIX-02 会话选中与滚动解耦

目标：

- 点击会话只更新选中状态或受控路由状态
- 不再触发浏览器锚点滚动

验收：

- 点击会话后 `scrollY` 不会异常跳变
- 详情面板更新正常
- 刷新后仍能恢复目标会话

### FIX-03 成本语义修复

目标：

- 区分“真实 0 成本”和“未知/不可判定”

验收：

- 无可靠成本依据时不再显示 `$0.00`
- 真实 0 值时才显示 `$0.00`
- 总览和详情语义一致

## 4. 当前执行状态

- `FIX-01` 已完成
  浏览器模式默认不再默默回退整包内置样例；只有显式开启 demo 模式时才允许展示样例 sessions/configs/usage/runtime。
- `FIX-02` 已完成
  会话选中改成受控状态更新；在首页嵌入区不会再强制跳到 `#/sessions/...`，在 Sessions 路由里也不会再触发异常滚动。
- `FIX-03` 已完成
  `costUsd` 已区分真实 `0` 和未知成本；无可靠依据时不再显示 `$0.00`。

相关验证已经进入当前测试链：

- `web/src/lib/api.test.ts`
- `web/src/app.test.tsx`
- `web/src/components/session-detail.test.tsx`
- `web/src/components/usage-panel.test.tsx`
- `tests/e2e/open-session-manager.spec.ts`

## 5. TDD 要求

`FIX-01` 必须先有失败测试，再改生产代码。

最小测试覆盖：

1. `fetchDashboardSnapshot()` 在浏览器模式拿不到真实快照时，默认不能退回内置样例
2. `App` 在该场景下不能展示 `GitHub Copilot CLI` / `Factory Droid` 这些样例配置
3. 只有显式开启 demo 模式时，样例数据才允许出现

## 6. 非目标

这轮之外仍不顺手混改：

- hash 路由整体重构
- usage/cost 模型重构
- 视觉样式调整
- 配置写回

先把数据真值修好，再推进后两条。
