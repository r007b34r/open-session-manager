# open Session Manager Design

日期：2026-03-15

## 0. 当前假设

- 默认不在界面中直接明文展示 API key
- 所有敏感值默认脱敏，只展示来源、用途、指纹、最后修改时间和风险标签
- 删除动作默认走“导出/备份 -> 软删除 -> 审计记录”，不直接硬删
- Win11 对 OpenCode 采用 WSL-first 方案；原生 Windows 仅在检测到可信本地路径时纳入支持

## 1. 方案选型

### 方案 A：纯 Electron + TypeScript

优点：

- 前后端统一用 TS，迭代快
- Web UI 形态天然成立
- 打包桌面应用简单

缺点：

- 文件系统治理、SQLite、跨 WSL/Windows 路径桥接、安全边界控制都不够硬
- 对大量 transcript 和大目录扫描的性能与内存占用不理想
- 敏感配置/删除动作的可信执行链不够强

### 方案 B：Rust 本地核心 + React Web UI + Tauri 打包

优点：

- Rust 负责扫描、解析、索引、删除、归档、审计，性能和可靠性更好
- React 负责可视化，仍然满足网页形态
- Tauri 体积小、安全边界更清晰，适合 Win11/Linux 桌面分发
- 便于增加 WSL companion collector、文件锁、加密缓存、后台 watcher

缺点：

- 学习和工程复杂度更高
- 前后端接口设计要更严格

### 方案 C：纯本地服务 + 浏览器访问

优点：

- 最贴近“网页可视化”
- 适合 headless 和远程访问

缺点：

- 本地认证、端口、浏览器访问权限、服务生命周期管理会更麻烦
- 普通用户的安装体验不如桌面应用

### 推荐

推荐采用“方案 B 的混合形态”：

- 核心做成 Rust 本地服务
- UI 做成 React Web 前端
- 默认以 Tauri 桌面应用分发
- 同时保留 `serve` 模式，允许在 Linux 或 WSL 中启动本地 Web 控制台

这样既满足网页可视化，也保证本地治理动作足够可靠。

## 2. 产品边界

### v1 必做

- 检测 Codex、Claude Code、OpenCode 的安装、配置、会话和关键路径
- 统一索引 session、config、credential metadata
- 展示主题、摘要、进度、最后活跃时间、仓库上下文、价值评分、风险评分
- 导出 Markdown
- 归档、软删除、恢复、审计
- 配置风险盘点：危险权限、第三方 relay、非官方 endpoint、重复 key 引用、失效配置

### v1 明确不做

- 远程云同步
- 多用户协作
- 自动上传 transcript 到外部云端做分析
- 默认全盘扫描整个 home 目录寻找任意 secret

## 3. 总体架构

```text
Tauri Shell / Browser
        |
        v
React UI
        |
        v
Rust Local API (HTTP/WebSocket + command bridge)
        |
        +-- Discovery Engine
        +-- Adapter Runtime
        +-- Session Intelligence Pipeline
        +-- Config/Credential Auditor
        +-- Export/Archive/Delete Engine
        +-- Audit Logger
        |
        +-- Local Metadata DB (SQLite)
        +-- Encrypted Cache / Snapshots
        +-- Native FS + WSL Companion Collectors
```

### 核心原则

- 原始数据尽量不复制，只做路径引用、内容哈希和派生索引
- 破坏性动作一定经过策略层，不让 UI 直接删文件
- Session 解析和配置解析都走 adapter
- 敏感内容、AI 总结、删除建议全部附带置信度和证据链

## 4. 平台支持策略

### Linux

- 直接扫描用户 home 下的已知助手目录
- 使用本地 watcher 增量更新

### Win11

- 扫描原生 Windows 助手路径
- 枚举 WSL 发行版
- 对每个发行版使用 `wsl.exe -d <name> --exec` 拉起 companion collector
- collector 返回标准化 JSON，主进程统一汇总

### 为什么不用直接读 `\\\\wsl$`

- 可行但不稳
- 大量文件扫描时性能和权限细节都更差
- 无法自然复用 Linux 侧路径探测和工具命令

结论：WSL companion collector 是更稳的主路径。

## 5. 适配器设计

每个助手一个 adapter，统一实现这些能力：

- `detect_installations()`
- `discover_config_roots()`
- `discover_session_roots()`
- `parse_session()`
- `parse_config()`
- `enumerate_credentials()`
- `archive_session()`
- `delete_session()`
- `restore_session()`
- `health_check()`

### 首批 adapter

- `codex`
- `claude_code`
- `opencode`

### 第二批候选

- `gemini_cli`
- `aider`
- `cursor_agent`
- `copilot_cli`
- `cline`
- `kimi_cli`

## 6. 统一数据模型

### Installation

- 助手名、版本、执行路径、平台、来源环境

### SessionRecord

- `session_id`
- `assistant`
- `environment`：windows / linux / wsl:<distro>
- `project_path`
- `source_path`
- `started_at`
- `ended_at`
- `last_activity_at`
- `message_count`
- `tool_count`
- `status`
- `raw_format`
- `content_hash`

### SessionInsight

- `title`
- `topic_labels`
- `summary`
- `progress_state`
- `progress_percent`
- `value_score`
- `stale_score`
- `garbage_score`
- `risk_flags`
- `confidence`

### ConfigArtifact

- 文件路径、作用域、来源层级、provider、model、base_url、权限、mcp、skills、agents

### CredentialArtifact

- `kind`
- `provider`
- `location`
- `source_type`：env / json / toml / shell_profile / keyring_ref
- `masked_value`
- `fingerprint`
- `official_or_proxy`
- `last_modified_at`

### AuditEvent

- 谁发起
- 何时发起
- 对哪个 session/config 生效
- 动作前摘要
- 动作后结果
- 失败原因

## 7. Session Intelligence Pipeline

要实现“识别真实主题、内容、进度”，不能只做字符串搜索，建议采用两层管线：

### 第一层：确定性提取

- 提取首个用户目标
- 提取 assistant 自带 summary/title
- 提取仓库路径、分支、模型、耗时、文件改动、命令执行、错误事件
- 提取 TodoWrite/TODO、工具调用、失败重试、最终状态

### 第二层：派生理解

- 根据用户目标 + assistant 总结 + 文件/命令上下文生成 `title`
- 根据 TODO 完成度、最近错误、是否存在最终交付语义生成 `progress_state`
- 根据代码改动、问题解决程度、信息密度、复用潜力生成 `value_score`
- 根据时效、重复度、空内容、极短会话、无结果退出生成 `garbage_score`

### 置信度机制

所有派生字段都带：

- `confidence`
- `evidence_refs`

如果置信度低，再允许用户显式启用本地 AI 总结增强。

## 8. 配置与密钥治理

这个模块是同类项目普遍缺失、但你需求里非常核心的一块。

### 默认扫描范围

- 助手官方配置目录
- 项目级 `.claude` / `.codex` / `.opencode`
- 已知 shell profile 中与助手相关的变量
- MCP / provider / endpoint / auth helper 配置

### 风险识别

- 非官方 `base_url`
- 第三方 relay / gateway
- 明文写死的 API key
- 同一 key 被多个助手共享
- 权限过宽
- “永不确认”类危险执行设置
- 过期配置、孤儿配置、重复 provider 定义

### 展示策略

- 默认只显示脱敏值和指纹
- 单独的“敏感查看”模式需要二次确认
- 导出报告默认不包含明文 secret

## 9. 导出、归档、删除工作流

### 推荐流程

1. 选中候选会话
2. 自动生成价值预览
3. 导出 Markdown
4. 选择归档到本地冷存储或项目知识库目录
5. 执行软删除
6. 写入审计日志

### Markdown 导出结构

- YAML frontmatter
- 会话标题
- 原始目标
- 关键结论
- 当前进度
- 关键命令与错误
- 关键文件变更
- 可复用片段
- 原始 transcript 附录或引用路径

### 删除策略

- 软删除：移入应用自己的 quarantine 目录或系统回收站
- 可恢复：保留 manifest 和原始路径
- 硬删除：仅在用户显式确认后执行

## 10. UI 设计

### 10.1 总览页

- 助手安装概况
- 活跃/陈旧/高价值/高风险会话数量
- 磁盘占用
- 风险配置计数

### 10.2 Session Explorer

- 按助手、项目、时间、状态、价值、风险过滤
- 支持全文检索和 faceted search
- 提供“建议清理队列”

### 10.3 Session Detail

- 标题、真实主题、进度条、最后活跃时间
- 时间线
- 关键命令/错误/文件
- 导出和归档动作
- 删除建议解释

### 10.4 Config Center

- 配置层级对比
- key 与 endpoint 面板
- relay 风险提示
- MCP/skills/agents 概览

### 10.5 Audit Center

- 扫描日志
- 导出日志
- 删除/恢复日志

## 11. 安全要求

- 所有解析和索引默认本地执行
- 不把 transcript 和 secret 上传到云端
- 本地缓存加密
- 破坏性动作必须记录不可抵赖日志
- UI 不直接持有删除权限，必须通过策略化命令执行
- 导出时做 secret redaction

## 12. 验证指标

### 功能指标

- 三个首批 adapter 可稳定识别安装与会话
- 高价值会话导出为 Markdown 后可读性达标
- 软删除和恢复链路完整

### 质量指标

- 大目录扫描不阻塞 UI
- 增量索引足够快
- 同一 session 多次重扫结果稳定

### 安全指标

- 默认界面不泄露明文 secret
- 所有 destructive actions 有日志

## 13. 分阶段路线图

### Phase 0

- 仓库、架构、文档、原型数据模型

### Phase 1

- Codex / Claude Code / OpenCode adapter
- 统一索引
- Session Explorer
- Markdown 导出

### Phase 2

- Config Center
- Credential 风险审计
- 软删除 / 恢复 / 审计中心

### Phase 3

- WSL companion collector
- 更深的价值判断和垃圾会话识别
- 更多助手适配器

## 14. 最终建议

产品定位建议明确成：

> 本地优先、多助手、可审计的会话与配置治理平台。

不要把第一版做成：

- 单纯 transcript viewer
- 单纯 launcher
- 单纯工作台皮肤

真正的差异化在于：

- 多助手统一治理
- 删除前内容提炼
- 配置与密钥风险识别
- Win11 + Linux + WSL 一体化支持
