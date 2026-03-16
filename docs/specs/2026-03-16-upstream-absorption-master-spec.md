# Upstream Absorption Master Spec

**Date:** 2026-03-16  
**Owner:** `r007b34r`  
**Branch:** `feat/usability-clarity`

---

## 1. Product Target

OSM 的目标不再只是“看一眼本地 transcript 列表”，而是成为一个真正可用的本地治理台：

- 发现并解析多种终端代码助手会话
- 识别真实主题、进度、价值、风险、最后活跃时间
- 在删除前完成导出、handoff、隔离和恢复
- 发现并审计各助手配置、provider、base URL、权限、MCP 和敏感键
- 继续扩展到 token / cost / usage、搜索、恢复控制、自动化接口和 worktree 编排

## 2. Evidence Base

本 spec 只基于两类证据：

1. 已真实拉取到本地 `third_party/upstreams/mirrors/` 的上游仓库
2. 2026-03-16 在线检索到的补充竞品线索

当前已纳入本地镜像并参与分析的上游：

- `jazzyalex/agent-sessions`
- `kbwo/ccmanager`
- `Dimension-AI-Technologies/Entropic`
- `milisp/codexia`
- `farion1231/cc-switch`
- `Dicklesworthstone/coding_agent_session_search`
- `d-kimuson/claude-code-viewer`
- `daaain/claude-code-log`
- `lulu-sk/CodexFlow`
- `yoavf/ai-sessions-mcp`
- `smtg-ai/claude-squad`
- `siteboon/claudecodeui`
- `junhoyeo/tokscale`
- `coder/agentapi`
- `endorhq/rover`
- `kevinelliott/agentpipe`
- `autohandai/commander`
- `pchalasani/claude-code-tools`
- `vultuk/claude-code-web`
- `ssdeanx/Gemini-CLI-Web`
- `sugyan/claude-code-webui`
- `udecode/dotai`
- `ChristopherA/claude_code_tools`

补充在线检索到但本轮镜像未成功的细分参考：

- `pangjiuzui/cxresume`

## 3. Capability Domains

上游能力已经收敛成 8 条产品主线：

1. 多助手会话发现与解析
2. transcript viewer / handoff / cleanup
3. 搜索、索引与知识复用
4. 配置、provider、MCP、skills、prompts 治理
5. token / cost / usage analytics
6. worktree、并行执行、会话恢复与控制
7. HTTP / MCP / remote / headless 平台化
8. diagnostics / repair / fixtures / drift / quality automation

## 4. Real Absorption Ledger

### 4.1 已真实落地到 OSM 的能力

#### `jazzyalex/agent-sessions`

- 已吸收：`Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 会话适配思路
- 已落地：clean-room 会话发现与解析，以及搜索结果呈现的工作台方向
- 未落地：active cockpit、后台索引、schema drift 治理、resume 控制

#### `daaain/claude-code-log`

- 已吸收：更完整的 Markdown 导出分节
- 已落地：Markdown export、transcript highlights、todo snapshot
- 未落地：HTML 导出、批量导出

#### `d-kimuson/claude-code-viewer`

- 已吸收：viewer 风格详情面板、todo evidence
- 已落地：更完整的 session detail
- 未落地：真实会话控制、git diff、terminal、MCP viewer

#### `ChristopherA/claude_code_tools`

- 已吸收：session handoff / resume brief 思路
- 已落地：Markdown `Session Handoff`
- 未落地：cleanup checklist、resume artifact 标准化、worktree helper 工具链

#### `farion1231/cc-switch`

- 已吸收：`Gemini CLI / OpenClaw` 配置路径、provider、base URL、auth mode、风险审计
- 已落地：`Gemini CLI / OpenClaw` 配置治理读取和风险展示
- 未落地：写回编辑、provider presets、MCP 管理、skills/prompts、proxy takeover、cost dashboard

#### `junhoyeo/tokscale`

- 已吸收：多助手 usage/cost 解析样式与字段模型
- 已落地：本地 usage/cost 面板，覆盖 `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw`
- 未落地：定价同步、排行榜、贡献图、外部同步源

#### `yoavf/ai-sessions-mcp`

- 已吸收：本地 lexical ranking、context snippet、match source 的搜索呈现思路
- 已落地：Web 工作台中的加权本地搜索、命中片段和来源标签
- 未落地：MCP `list/search/get`

### 4.2 已研究但仍未真正落地的高价值能力

#### `coding_agent_session_search`

- 待吸收：BM25 / semantic / hybrid search、大库索引和 robot mode

#### `kbwo/ccmanager`

- 待吸收：worktree 生命周期、命令预设、fallback、状态探测

#### `claude-squad`

- 待吸收：attach / detach / pause / resume

#### `agentapi`

- 待吸收：HTTP / SSE 控制层

#### `ai-sessions-mcp`

- 待吸收：MCP `list/search/get`

#### `Entropic`

- 待吸收：diagnostics / repair / Git 全局视图

#### `siteboon/claudecodeui`

- 待吸收：remote/self-hosted、移动端响应式、插件系统

#### `CodexFlow`

- 待吸收：更强的 Win11 + WSL 双态桥接、resume 和 usage center

#### `agentpipe`

- 待吸收：doctor / health、HTML export、Prometheus metrics

## 5. Current OSM Baseline

### 5.1 已有

- 会话支持：`Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw`
- 配置审计：`Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw`
- 会话详情：主题、摘要、进度、价值、风险、高亮、todo
- 本地加权搜索、命中片段和来源标签
- usage/cost analytics：`Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw`
- Markdown 导出、handoff、软删除、恢复
- 审计历史持久化
- Web / Tauri 共用 UI
- 中英文切换、自动语言、主题切换、导出目录偏好

### 5.2 还不够

- 已有 usage/cost 面板，但仍缺 pricing lookup 和趋势分析
- 缺 `GitHub Copilot CLI / Factory Droid` 配置治理
- 已有加权本地搜索预览，但仍缺 BM25 / semantic / API search
- 缺会话恢复 / attach / pause / resume
- 缺 worktree 编排
- 缺 MCP / HTTP / headless
- 缺 provider/MCP/skills/prompts 一体化治理

## 6. This Tranche

本轮不做“空泛全吸收”承诺，而是聚焦一条必须真实落地的能力线：

### 6.1 必做

- 基于 `tokscale` 的 usage/cost 解析思路，给 OSM 增加本地 usage analytics
- 覆盖至少这些已支持且有公开格式样例的助手：
  - `Codex`
  - `Claude Code`
  - `OpenCode`
  - `Gemini CLI`
  - `OpenClaw`
- 在 Rust snapshot、CLI snapshot、Web UI 中都能看到 usage/cost 数据
- 用 fixture + TDD 固化格式和回归

### 6.2 同步做

- 写出总 spec 和总 plan
- README / 支持矩阵 / 竞品差距分析同步更新

### 6.3 本轮暂不承诺

- token 定价在线拉取
- 多账号 usage 聚合
- `GitHub Copilot CLI / Factory Droid` 完整配置治理
- HTTP / MCP 平台化
- 会话恢复控制

## 7. Acceptance Criteria

满足以下条件，本轮才能宣称“真实吸收了一条新的竞品能力线”：

1. Rust 单测先失败再转绿，覆盖 usage 解析
2. `cargo test --test cli_snapshot` 能看到 usage 字段
3. Web 单测能看到 usage/cost 面板或会话 usage 细节
4. `scripts/verify.ps1` 通过
5. 文档明确区分：
   - 已真实吸收
   - 已研究未实现
   - reference-only / 许可证受限

## 8. Release Language Constraint

发布口径必须诚实：

- 可以说：OSM 正在从会话整理器升级为多助手治理平台
- 不可以说：已经完全超越所有竞品
- 不可以说：已经完整吸收全部能力

本 spec 的目的正是把“真实吸收”和“仅研究”彻底分开。
