# 2026-03-16 本地代码审计

日期：2026-03-16  
工作树：`feat/usability-clarity`

## 审计范围

- Rust 核心与导出/清理链路
- Web/Tauri 前端交互
- upstream research / release 文档
- 统一验证脚本与本地构建链

## 已确认并处理

### 1. Upstream intake 校验已恢复

`scripts/verify.ps1` 之前会在 upstream intake 测试上失败。根因不是生成器坏了，而是 `third_party/upstreams/catalog.json` 扩容后，`jazzyalex/agent-sessions` 这一项被改成了浅条目，测试还在断言旧的深度审计字段。

处理结果：

- 补回了 `agent-sessions` 的真实源码路径、关注点和吸收映射
- 重新生成了 research / attribution / intake manifest
- 统一验证重新通过

### 2. Markdown 导出已补上会话交接摘要

之前的导出更像审计备忘，不够适合“删前保留核心价值”。现在导出里新增了 `Session Handoff` 段落，包含：

- `Next focus`
- `Open tasks`
- `Completed tasks`
- `Resume cue`

它是从 transcript todo、highlights 和当前 summary 里做的确定性派生，没有额外引入模型调用。

## 仍然存在的明显问题

### P0

1. `docs/research/2026-03-16-full-competitor-gap-analysis.md` 一度把已经实现的 `Gemini / Copilot / Factory / OpenClaw` 适配器又写回“缺失”。
现在已修正主要矛盾，但这类总表文档仍然需要继续人工复核，不能完全依赖一次性生成或大段改写。

2. 竞品治理目录还不够深。
这次已经把 `agent-sessions` 补成深条目，并新拉了 `ChristopherA/claude_code_tools`、`ssdeanx/Gemini-CLI-Web`、`sugyan/claude-code-webui` 到本地，但其余新增仓库还有不少只是“概览级”记录，没有全部补齐 verified paths 和 integration targets。

### P1

1. `ssdeanx/Gemini-CLI-Web` 的许可证口径冲突。
`LICENSE` 是 GPL-3.0，README 也明确提到 GPL 来源链，但 `package.json` 却写 MIT。这个仓库当前不能当成可直接吸收的代码来源，只能 reference-only。

2. `npm --prefix web run test` 过程中仍会出现 `--localstorage-file` 无效路径警告。
它不影响当前测试通过，但说明本地测试环境或上游依赖的启动参数还有噪声，后面最好把来源追出来。

3. 统一验证脚本仍只做桌面 debug build。
发布前还需要补 release build smoke test，尤其是 Windows 下“不要弹终端窗口”这个诉求，不能只靠 debug 构建结果判断。

## 下一轮优先级建议

1. 先做全文检索和 snippet 结果页。这是 OSM 目前最明显的功能断层。
2. 再补 `Gemini / Copilot / Factory / OpenClaw` 的配置治理，避免“会话支持了，配置还看不到”。
3. 然后推进 worktree 和轻量远程壳层，这两条会直接决定 OSM 能不能超过现有竞品。
