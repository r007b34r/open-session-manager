# Web-First Config Governance Phase 2 Spec

**Date:** 2026-03-16
**Owner:** `r007b34r`
**Scope:** `Web-first launcher decision` + `Gemini CLI / OpenClaw config governance`

---

## 1. Problem

OSM 现在虽然已经有浏览器端 React UI，但主运行叙事仍然偏向 `Tauri/EXE`。这和当前用户偏好不一致，也和一批竞品已经转向的“本地服务 + 浏览器”方向不一致。

同时，OSM 当前真正可用的配置治理只覆盖：

- `Codex`
- `Claude Code`
- `OpenCode`

这意味着用户要求的“把所有编程助手配置解析、预览、可视化修改”目前只实现了很小的一部分。

## 2. Upstream Evidence

本轮以真实本地镜像和最新公开仓库页为依据，重点参考：

- `farion1231/cc-switch`
  - 价值：`Gemini CLI`、`OpenClaw` 配置治理的最强参考
  - 已确认：
    - `Gemini` 使用 `~/.gemini/.env` 与 `~/.gemini/settings.json`
    - `OpenClaw` 使用 `~/.openclaw/openclaw.json`
    - 具备 provider、base URL、API key、MCP、OpenClaw tools/env/defaults 的结构化管理
- `sugyan/claude-code-webui`
  - 价值：证明“浏览器壳 + 历史加载 + 轻量后端”完全可行
- `vultuk/claude-code-web`
  - 价值：证明“远程浏览器壳 + 会话持久化 + API”是有效产品方向
- `ssdeanx/Gemini-CLI-Web`
  - 价值：证明 Gemini 方向已经存在 Web 工作台产品形态
  - 约束：许可证口径冲突，当前只能 reference-only

## 3. Product Decisions

### 3.1 OSM 运行形态改为双模，但以 Web-first 为主

对当前用户更合理的默认运行方式应为：

- 主路径：脚本启动本地 snapshot + Web 页面
- 可选路径：Tauri 桌面壳

也就是说：

- `Web-first` 是默认体验
- `Tauri` 退到可选桌面壳，不再是唯一主路径

### 3.2 Phase 2 真实落地目标

本轮必须真实达成：

1. `Gemini CLI` 配置能被发现、审计、展示。
2. `OpenClaw` 配置能被发现、审计、展示。
3. 配置面板至少能看到更接近“真实预览”的字段，而不是只剩路径和风险。
4. README / 支持矩阵 / 竞品分析明确区分：
   - 已真实吸收
   - 已研究但未实现
   - 受许可证约束只能参考

## 4. Non-Goals

本轮不承诺：

- `GitHub Copilot CLI` 与 `Factory Droid` 的完整配置写回
- token/cost analytics
- 会话恢复 / attach / pause / resume
- 全文搜索 / BM25 / 语义搜索
- 远程多用户部署

## 5. Functional Requirements

### 5.1 Gemini CLI Config Audit

OSM 必须支持：

- 发现 `~/.gemini/settings.json`
- 读取同级 `.env`
- 提取：
  - provider
  - model
  - base URL
  - API key 或 OAuth 模式
  - `security.auth.selectedType`
  - session retention / MCP 相关结构
- 识别风险：
  - `third_party_base_url`
  - `third_party_provider`
  - `missing_primary_secret`（非 OAuth 模式时）

### 5.2 OpenClaw Config Audit

OSM 必须支持：

- 发现 `~/.openclaw/openclaw.json`
- 以 JSON5 解析
- 提取：
  - provider ID
  - base URL
  - API key
  - 默认模型 / fallback
  - tools profile
  - env/tools/agents defaults 的存在性
- 识别风险：
  - `third_party_base_url`
  - `third_party_provider`
  - `dangerous_permissions`
  - `config_parse_failed`

### 5.3 Config Preview Upgrade

配置卡片至少要新增：

- model
- 更明确的 provider 命名
- 更自然的 “未配置 / OAuth / 已检测” 状态文案

即使暂时还不支持写回，也必须先把“能看懂”这一步做对。

## 6. Architecture

### 6.1 保持当前 Rust 审计管线

继续复用：

- `discover_known_config_targets`
- `audit_config`
- `build_config_records`
- `DashboardSnapshot.configs`

新增助手通过 clean-room 审计分支接入，不重写整套 dashboard。

### 6.2 Web-first 先做方案，不在本轮和配置治理强耦合

Web-first 启动链与配置治理要拆成两条线：

- 本轮代码主落点：配置治理扩面
- 本轮方案落点：Web-first 启动器 spec/plan

避免再次出现“架构大改和能力补齐混在一起，最后两边都做浅”的问题。

## 7. Acceptance Criteria

满足以下条件才可宣称本轮 Phase 2 完成：

1. `cargo test --lib audit::tests` 通过
2. fixture snapshot 能包含 `Gemini CLI` 和 `OpenClaw` 配置记录
3. Web 配置面板能显示新增配置记录与 model 字段
4. `scripts/verify.ps1` 通过
5. README / 支持矩阵 / 竞品差距分析完成同步

## 8. Release Impact

本轮完成后，OSM 在“配置治理”维度会从：

- `Codex / Claude Code / OpenCode`

提升为：

- `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw`

这不是最终目标，但会第一次把 OSM 从“只会看部分历史”的工具，推向“真正开始治理多助手配置”的方向。
