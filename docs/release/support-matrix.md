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
| Web 总览 / 会话 / 配置 / 审计 UI | 已实现 | 桌面端与浏览器端共用同一套 React UI |
| 中英文切换 | 已实现 | 默认按浏览器语言自动切换 |
| 真实 snapshot CLI | 已实现 | `cargo run -p agent-session-governance-core -- snapshot` |
| 前端真实 snapshot 优先加载 | 已实现 | 失败时回退到 fixture |
| Tauri 桌面运行时 | 已实现 | 无参数启动进入桌面窗口，前端可直接调用 Rust 原生命令 |
| 导出优先清理守卫 | 已实现 | 后端拒绝未导出会话的软删除，前端按钮同步禁用 |
| 持久化审计数据库读取 | 已实现 | snapshot 可读取本地持久化审计历史 |

## 平台支持

| 平台 | 当前状态 | 说明 |
| --- | --- | --- |
| Windows 11 | 已验证调试构建 | `cargo test`、`scripts/verify.ps1`、`npm --prefix web run tauri:build -- --debug` 已在当前环境通过 |
| Linux | 核心已验证 | 会话 / 配置路径模型与 fixtures 已覆盖，桌面构建尚未完成实机验证 |
| WSL | 部分实现 | 当前有路径模型和 UI 表达，尚未完成 companion collector |

## 发布前仍需补齐

| 项目 | 当前状态 | 风险 |
| --- | --- | --- |
| Linux 桌面实机回归 | 未完成 | 当前还没有 Linux 环境下的 Tauri 构建与真实助手目录回归证据 |
| 发布安装包与签名 | 未完成 | 目前只有 Windows 调试构建产物，没有 MSI / AppImage / deb / 签名流程 |
| 配置修改 / 删除 UI | 未完成 | 当前配置中心以识别和审计为主，尚未把安全修改流程接到真实文件写回 |
| 恢复与隔离区管理 UI | 未完成 | Rust 已支持 restore，但前端还没有完整的恢复工作流和隔离区视图 |
| WSL companion collector | 未完成 | WSL 仍主要停留在路径模型与展示层，缺少跨环境真实采集闭环 |

## 当前验证证据

- Rust 测试：`cargo test`
- Snapshot CLI 测试：`cargo test --test cli_snapshot`
- Web 单测：`npm --prefix web run test`
- Web 构建：`npm --prefix web run build`
- Windows 桌面调试构建：`npm --prefix web run tauri:build -- --debug`
- Playwright：`npm --prefix web run e2e`
- 统一入口：`powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
