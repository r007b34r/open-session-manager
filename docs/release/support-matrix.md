# 支持矩阵

日期：2026-03-15
发布：`v0.2.1 Public Preview`

## 本次公开发布范围

| 维度 | 当前状态 | 说明 |
| --- | --- | --- |
| Codex 配置发现 | 已实现 | 支持用户级配置路径发现与审计 |
| Codex 会话解析 | 已实现 | 支持 `~/.codex/sessions` JSONL rollout |
| Claude Code 配置发现 | 已实现 | 支持用户级 `settings.json` |
| Claude Code 会话解析 | 已实现 | 支持 `~/.claude/projects/**/*.jsonl` |
| OpenCode 配置发现 | 已实现 | 支持全局 `opencode.json` |
| OpenCode 会话解析 | 已实现 | 支持 `~/.local/share/opencode` storage |
| Transcript digest / todo snapshot | 已实现 | 支持 transcript highlights、Claude todo 提取与会话详情展示 |
| Markdown 导出 | 已实现 | Rust actions 已有测试覆盖 |
| 软删除 / 恢复 | 已实现 | Rust actions 已有测试覆盖，恢复边界已加固 |
| 审计事件写入 | 已实现 | 当前覆盖导出、软删除、恢复 |
| Web 总览 / 会话 / 配置 / 审计 UI | 已实现 | 桌面端与浏览器端共用同一套 React UI |
| 中英文切换 | 已实现 | 默认按浏览器语言自动切换 |
| 真实 snapshot CLI | 已实现 | `cargo run -p open-session-manager-core -- snapshot` |
| 前端真实 snapshot 优先加载 | 已实现 | 失败时回退到 fixture |
| Tauri 桌面运行时 | 已实现 | 无参数启动进入桌面窗口，前端可直接调用 Rust 原生命令 |
| 导出优先清理守卫 | 已实现 | 后端拒绝未导出会话的软删除，前端按钮同步禁用 |
| 持久化审计数据库读取 | 已实现 | snapshot 可读取本地持久化审计历史 |
| upstream intake pipeline | 已实现 | 支持 catalog、研究索引、发布致谢和镜像规划产物 |

## 平台支持

| 平台 | 当前状态 | 说明 |
| --- | --- | --- |
| Windows 11 | 已验证 | `cargo test --lib`、`cargo test --test cli_snapshot`、`scripts/verify.ps1`、`npm --prefix web run tauri:build -- --debug` 已在当前环境通过 |
| Linux | 核心能力预览 | 会话 / 配置路径模型与 fixtures 已覆盖，桌面构建尚未完成实机验证 |
| WSL | 能力预览 | 当前有路径模型、发现表达和 UI 展示，尚未完成 companion collector |

## 当前已吸收的开源能力

| 上游项目 | 当前状态 | 已落地内容 |
| --- | --- | --- |
| `daaain/claude-code-log` | 已吸收 | 更丰富的 Markdown 导出分节、transcript highlights、Claude todo snapshot |
| `d-kimuson/claude-code-viewer` | 已吸收 | viewer 风格 transcript detail 面板、session todo evidence 展示 |
| `jazzyalex/agent-sessions` | 已研究 | 本地索引、搜索、analytics 方向 |
| `lulu-sk/CodexFlow` | 已研究 | Windows / WSL / 多助手工作区方向 |
| `Dimension-AI-Technologies/Entropic` | 已研究 | provider-aware 治理控制台方向 |
| `yoavf/ai-sessions-mcp` | 已研究 | 后续 MCP / headless 暴露方向 |
| `Dicklesworthstone/coding_agent_session_search` | Reference-only | 仅吸收连接器覆盖策略与产品启发，不直接吸收代码 |

## 当前不在 v0.2.1 承诺范围内

| 项目 | 当前状态 | 说明 |
| --- | --- | --- |
| Linux 桌面实机回归 | 未纳入本版承诺 | 当前还没有 Linux 环境下的 Tauri 构建与真实助手目录回归证据 |
| 发布安装包与签名 | 未纳入本版承诺 | 目前以源码仓库与 Windows 可执行构建产物发布为主，没有 MSI / AppImage / deb / 签名流程 |
| 配置修改 / 删除 UI | 未纳入本版承诺 | 当前配置中心以识别和审计为主，尚未接上真实文件写回 |
| 恢复与隔离区管理 UI | 未纳入本版承诺 | Rust 已支持 restore，但前端还没有完整恢复工作流和隔离区视图 |
| WSL companion collector | 未纳入本版承诺 | WSL 仍主要停留在路径模型与展示层，缺少跨环境真实采集闭环 |

## 发布后优先项

- 真实 Win11 + WSL 多发行版样本回放
- 符号链接 / 目录联接逃逸专项测试
- 超大历史库性能压测
- 发布包 smoke test

## 当前验证证据

- Rust 测试：`cargo test --lib`
- Snapshot CLI 测试：`cargo test --test cli_snapshot`
- Clippy：`cargo clippy --all-targets --all-features -- -D warnings`
- Web 单测：`npm --prefix web run test`
- Web 构建：`npm --prefix web run build`
- Windows 桌面调试构建：`npm --prefix web run tauri:build -- --debug`
- Playwright：`npm --prefix web run e2e`
- 统一入口：`powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
