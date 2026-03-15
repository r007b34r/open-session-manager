# 支持矩阵

日期：2026-03-15

## 当前实现范围

| 维度 | 当前状态 | 说明 |
| --- | --- | --- |
| Codex 配置发现 | 已实现 | 支持用户级配置路径发现与审计 |
| Codex 会话解析 | 已实现 | 支持 `~/.codex/sessions` JSONL rollout |
| Claude Code 配置发现 | 已实现 | 支持用户级 `settings.json` |
| Claude Code 会话解析 | 已实现 | 支持 `~/.claude/projects/**/*.jsonl` |
| OpenCode 配置发现 | 已实现 | 支持全局 `opencode.json` |
| OpenCode 会话解析 | 已实现 | 支持 `~/.local/share/opencode` storage |
| Markdown 导出 | 已实现 | Rust actions 已有测试覆盖 |
| 软删除 / 恢复 | 已实现 | Rust actions 已有测试覆盖 |
| 审计事件写入 | 已实现 | 当前覆盖导出、软删除、恢复 |
| Web 总览 / 会话 / 配置 / 审计 UI | 已实现 | 仍以本地优先读取 snapshot 为主 |
| 中英文切换 | 已实现 | 默认按浏览器语言自动切换 |
| 真实 snapshot CLI | 已实现 | `cargo run -p agent-session-governance-core -- snapshot` |
| 前端真实 snapshot 优先加载 | 已实现 | 失败时回退到 fixture |

## 平台支持

| 平台 | 当前状态 | 说明 |
| --- | --- | --- |
| Windows 11 | 开发中 | 已有路径发现、测试夹具、前端验证 |
| Linux | 开发中 | 已有 OpenCode 路径与 storage fixture 验证 |
| WSL | 部分实现 | 当前有路径模型和 UI 表达，尚未完成 companion collector |

## 发布前仍需补齐

| 项目 | 当前状态 | 风险 |
| --- | --- | --- |
| Tauri 桌面桥接 | 未完成 | 目前是 Rust CLI + Web 原型，非完整桌面分发 |
| 持久化审计数据库位置策略 | 未完成 | 当前 snapshot 尚未读取持久化审计历史 |
| 真实 Win11 / Linux 集成回归 | 未完成 | 当前主要依赖 fixtures 和本地开发验证 |
| 默认安全确认流 | 未完成 | UI 侧仍需把“导出优先、再清理”做成强约束 |
| 发布安装包 | 未完成 | 尚无 MSI / AppImage / deb 等产物流程 |

## 当前验证证据

- Rust 测试：`cargo test`
- Snapshot CLI 测试：`cargo test --test cli_snapshot`
- Web 单测：`npm --prefix web run test`
- Web 构建：`npm --prefix web run build`
- Playwright：`npm --prefix web run e2e`
- 统一入口：`powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
