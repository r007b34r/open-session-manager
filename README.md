# Agent Session Governance

本项目面向 Win11 和 Linux 用户，目标是把本地终端代码助手的会话、配置、密钥引用和清理动作统一纳入一个本地优先的治理平台。

首期目标聚焦：

- 识别本地安装的 `Codex`、`Claude Code`、`OpenCode`，并为后续扩展更多助手预留适配器接口
- 读取原生会话记录、配置文件、日志和关键运行元数据
- 在网页/桌面界面中展示真实主题、进度、价值密度、风险等级和最后活跃时间
- 在删除前先导出高价值会话为 Markdown，并支持归档、迁移、软删除和审计日志
- 默认对敏感配置和 API key 做脱敏处理，只在显式授权下进行更深层查看或修改

当前仓库先完成了本地 `git` 初始化，以及调研/设计文档沉淀：

- [竞品与数据源分析](docs/research/2026-03-15-agent-session-landscape.md)
- [完整设计方案](docs/plans/2026-03-15-agent-session-governance-design.md)
- [实施计划](docs/plans/2026-03-15-agent-session-governance.md)
