# open Session Manager v0.3.0 Public Preview

这次不是 `v0.2.x` 那种补丁修修补补，而是第一次把 OSM 真正推到多助手会话治理台的轨道上。

## 本版重点

- 会话支持面从 3 个扩到 7 个：
  - `Codex`
  - `Claude Code`
  - `OpenCode`
  - `Gemini CLI`
  - `GitHub Copilot CLI`
  - `Factory Droid`
  - `OpenClaw`
- 配置审计扩到 7 个助手，`GitHub Copilot CLI / Factory Droid` 已补上用户级配置治理
- `GitHub Copilot CLI / Factory Droid` 现在会按会话项目路径带出项目级配置覆盖层
- 把一批真实竞品镜像拉到本地并纳入 catalog、研究索引和开源致谢，不再只有零散笔记
- 修掉会直接影响可用性的会话质量问题：
  - `Codex` 不再把 `AGENTS.md`、环境注入块误当真实主题
  - `Claude Code` 不再把纯 `file-history-snapshot` JSONL 当候选会话
- Sessions 列表现在会显示会话 ID，多个相近标题不再像同一条
- 支持 usage / cost analytics：
  - `Codex`
  - `Claude Code`
  - `OpenCode`
  - `Gemini CLI`
  - `OpenClaw`
- Sessions 搜索现在会做本地加权排序，展示命中片段和命中来源标签
- 导出目录设置、导出后路径显示、语言切换、主题切换继续保留
- Markdown 导出补上了 `Session Handoff`，会把 `Next focus / Open tasks / Resume cue` 一起写进去
- 新增三条本地镜像研究并纳入治理目录：
  - `ChristopherA/claude_code_tools`
  - `ssdeanx/Gemini-CLI-Web`
  - `sugyan/claude-code-webui`

## 这版真正吸收了什么

已经真实落进代码的，不只是“研究过”：

- `jazzyalex/agent-sessions`
  - clean-room 吸收并重写了 `Gemini CLI`、`GitHub Copilot CLI`、`Factory Droid`、`OpenClaw` 适配器能力线
- `daaain/claude-code-log`
  - 延续 Markdown 导出、transcript highlights、todo snapshot 的思路
- `d-kimuson/claude-code-viewer`
  - 延续 viewer 风格详情面板和 todo evidence 呈现思路
- `ChristopherA/claude_code_tools`
  - 吸收 session closure / resume 的 brief 思路，补到 OSM 的 `Session Handoff` Markdown 导出
- `farion1231/cc-switch`
  - clean-room 吸收统一 provider/config 治理方向，以及 `Gemini CLI` 与 `OpenClaw` 配置治理中的路径、auth mode、provider/base URL 风险建模
- `endorhq/rover`
  - 吸收 `GitHub Copilot CLI` companion `mcp-config.json` 路径线索，补到 OSM 的 clean-room 配置审计里
- `junhoyeo/tokscale`
  - clean-room 吸收 usage / token / cost 字段模型和本地聚合面板
- `jazzyalex/agent-sessions`
  - 吸收本地搜索结果呈现、命中来源可视化的工作台方向
- `yoavf/ai-sessions-mcp`
  - 吸收本地 lexical ranking + snippet 的产品线索，先落到 Web 工作台

已经纳入本地镜像、研究索引和致谢体系的还包括：

- `kbwo/ccmanager`
- `farion1231/cc-switch`
- `junhoyeo/tokscale`
- `coder/agentapi`
- `endorhq/rover`
- `kevinelliott/agentpipe`
- `milisp/codexia`
- `siteboon/claudecodeui`
- `smtg-ai/claude-squad`
- `udecode/dotai`
- `autohandai/commander`
- `pchalasani/claude-code-tools`
- `vultuk/claude-code-web`
- `sugyan/claude-code-webui`
- `ssdeanx/Gemini-CLI-Web`

## 关于 `edition = "2024"`

`Cargo.toml` 继续使用 `edition = "2024"`，不是写错年份。

原因很简单：Rust edition 是语言版本，不是当前年份。当前本机 `cargo` 支持的 edition 仍然是：

- `2015`
- `2018`
- `2021`
- `2024`

`2026` 不是有效值，直接改会让仓库配置失效。

## 当前已实现的能力

- 7 个终端代码助手的本地会话发现与解析
- 7 个终端代码助手的配置审计读取与风险预览
- `GitHub Copilot CLI / Factory Droid` 的项目级配置发现
- `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw` 的 usage / cost 汇总
- 会话标题、摘要、进度、价值分、风险标记、最后活跃时间
- transcript highlights 与 Claude todo snapshot
- Sessions 页加权搜索、命中片段和来源标签
- `Session Handoff` Markdown 导出
- Markdown 导出、软删除、恢复、审计历史
- 中英文切换与跟随系统语言
- 浅色 / 深色 / 跟随系统主题
- Markdown 导出目录设置与导出路径显示
- Tauri 桌面运行时与浏览器 fallback
- upstream intake pipeline、研究索引与开源致谢

## 当前边界

以下内容不包含在 `v0.3.0 Public Preview` 承诺范围内：

- 大历史索引、BM25、语义搜索、hybrid ranking
- 会话恢复 / attach / pause / process control
- worktree 编排、多项目调度、容器隔离执行
- provider presets、共享配置片段、健康探测和自动切换
- pricing lookup、usage 趋势图、更多助手连接器
- MCP / HTTP / headless 自动化接口
- Linux 桌面实机回归
- 发布安装包与签名流程

## 验证结果

本版发布前已通过：

- `cargo test --lib`
- `cargo test --test cli_snapshot`
- `npm --prefix web run test`
- `npm --prefix web run build`

## 开源致谢

完整感谢名单和许可边界已收口到：

- `docs/release/open-source-attribution.md`
- `docs/research/upstreams/index.md`
- `third_party/upstreams/catalog.json`
