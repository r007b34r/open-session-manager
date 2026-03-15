# OSM 本地全量审计与进化路线

日期：2026-03-15

## 0. 17:00 后续修复状态

在本轮后续实现里，以下阻塞项已经被收口并重新验证：

- Windows release `exe` 已切换为 GUI 子系统，不再弹出终端
- Sessions 工作区已改成稳定的显式选中模型
- 桌面命令已改成异步 `spawn_blocking` 包装，避免把 Tauri UI 线程直接拖进全量扫盘
- `session_id` 导出/隔离路径已做净化
- `restore` 现在要求 manifest 与 payload 都位于受管隔离区内
- Markdown frontmatter 已改成安全转义输出

因此，下面的 findings 应理解为“审计过程中发现的问题 + 当前剩余风险”，而不是全部仍未修复。

## 1. 审计范围

本次审计基于以下证据源：

- Rust 核心、Tauri 桌面层、React 前端、测试与发布脚本的本地静态审阅
- 当前 worktree 下的本地验证命令
- GitHub 上同类开源项目的最新公开 README、功能描述与许可证信息
- 已结构化落地的 upstream intake catalog、研究索引与发布致谢产物

本轮确认过的本地验证状态：

- `cargo test` 通过
- `npm --prefix web run test` 通过
- `npm --prefix web run build` 通过
- `cargo clippy --all-targets --all-features -- -D warnings` 失败
- `scripts/verify.ps1` 在安装 `web` 依赖前失败，安装后才具备继续执行完整链路的条件

## 2. 当前实现的关键结论

这不是“不能用”的仓库，而是一个已经具备真实产品骨架、但还没有彻底跨过发布门槛的候选版本。

已经成立的部分：

- Win11 / Linux / WSL 的发现、解析、审计、导出、软删除和恢复主链路已经打通
- Codex、Claude Code、OpenCode 三类适配器已经具备真实 fixture 解析能力
- 前后端已经不是纯 demo，桌面命令会读取真实本地 snapshot
- 中英文切换与浏览器语言探测已经接上

还没有成立的部分：

- Windows 桌面 release 启动体验还不合格
- Sessions 工作区的交互模型还不够稳定
- 详情区布局没有达到可发布桌面软件的完成度
- 代码质量门禁还没有完全收口到“lint 级全绿”
- 上游代码还没有进入 clean-room 级继承实现，目前完成的是 intake 流程和许可证边界治理

## 3. Findings

### 严重级

1. 桌面端虽然已经异步化，但每次加载、导出和软删除后仍然会做全量重扫；数据量继续增大后，体感性能仍可能不足。
   影响：当本地 transcript 和配置变多时，桌面窗口会在主线程上出现明显冻结，发布后非常容易被判断为“不稳定”。
   证据：[`src-tauri/src/desktop.rs`](../../src-tauri/src/desktop.rs) 的 `load_dashboard_snapshot`、`export_session_markdown`、`soft_delete_session` 虽然已是异步命令，但动作完成后仍会再次调用全量 `build_local_dashboard_snapshot_with_audit(...)`。

### 高优先级

2. 会话列表虽然已经在前端按最近活跃时间排序，但后端 snapshot 仍未显式输出稳定排序，CLI/其他消费者可能和桌面端顺序不一致。
   影响：用户会感觉产品没有“理解当前工作状态”，默认打开的会话不稳定也不智能。
   证据：[`src-tauri/src/adapters/traits.rs`](../../src-tauri/src/adapters/traits.rs) 只保证文件路径排序；[`web/src/lib/api.ts`](../../web/src/lib/api.ts) 才做了二次排序。

3. restore 现在已经有受管根校验，但还没有配套的隔离区浏览器和恢复 UI，用户仍无法可视化审查 manifest。
   影响：后端更安全了，但桌面产品面仍缺少可操作的恢复治理入口。
   证据：当前前端仍没有隔离区独立页面，见 README 当前限制说明。

### 中优先级

4. 当前前端自动化主要验证功能流，仍缺少更多真实桌面窗口比例、长列表、超长路径和高 DPI 退化保护。
   影响：每次样式或布局调整都可能重新引入“能跑但难用”的问题。
   证据：[`tests/e2e/open-session-manager.spec.ts`](../../tests/e2e/open-session-manager.spec.ts) 目前只覆盖基本流程，没有覆盖多视口与选中态稳定性。

## 4. 竞品对照后的结论

OSM 现在已经比很多单助手 viewer 更接近“治理平台”，但和真正能压制竞品的状态相比，还差三层护城河：

### 第一层：稳定性护城河

- 启动即是 GUI，不弹终端
- 会话选中、搜索、导出、隔离都能稳定命中当前会话
- 宽窗口、窄窗口、高 DPI、Windows 缩放都不会把详情区拉坏

### 第二层：治理护城河

- 不只看 transcript，还看配置、relay、风险、敏感项、删除链路和审计证据
- 删除前自动形成“可复用的 Markdown 资产”，而不是只做 archive
- 能解释“为什么该删、为什么该留、为什么要迁移”

### 第三层：生态护城河

- 兼容更多 agent
- 兼容更多 transcript / config schema 版本
- 可吸收开源生态能力，但形成统一的 OSM 域模型和治理 UX

## 5. 可以直接吸收的竞品能力

### 可直接集成或重构吸收

- `jazzyalex/agent-sessions`
  - 价值：成熟的本地索引、搜索、analytics 设计
  - 许可证：MIT
  - 吸收方式：借鉴其统一索引和 session analytics 组织方式，必要时直接集成兼容层

- `lulu-sk/CodexFlow`
  - 价值：Windows / WSL 双环境、项目维度聚合、工作流组织
  - 许可证：Apache-2.0
  - 吸收方式：优先吸收 Windows/WSL 路径治理与项目上下文组织思路

- `d-kimuson/claude-code-viewer`
  - 价值：深度 session 详情、schema 校验、zero-data-loss 思路
  - 许可证：MIT
  - 吸收方式：吸收 transcript 解析、详情呈现和数据保真策略

- `daaain/claude-code-log`
  - 价值：高质量 Markdown / HTML 导出、摘要视图
  - 许可证：MIT
  - 吸收方式：吸收导出模板、结构化摘要和 usage 汇总思路

- `Dimension-AI-Technologies/Entropic`
  - 价值：多 provider GUI、维护动作入口
  - 许可证：MIT
  - 吸收方式：吸收 provider-aware navigation 与维护面板组织

- `yoavf/ai-sessions-mcp`
  - 价值：把本地会话暴露成可检索工具接口
  - 许可证：MIT
  - 吸收方式：后续可把 OSM 的会话治理能力反向暴露为 MCP 服务

### 只能借鉴，不能无脑吸收

- `Dicklesworthstone/coding_agent_session_search`
  - 价值：多助手覆盖面极广，统一 connector 经验很强
  - 限制：仓库带有附加使用限制，不适合直接把代码并入 OSM 再发布
  - 吸收方式：吸收 connector 覆盖策略、输入源枚举和导出思路，不直接拷源码

## 6. 要形成“碾压竞品”的产品路线

### Phase 1：发布级止血

- 修正 Windows GUI 启动
- 重做 Sessions 工作区交互模型
- 给详情区做非拉伸布局、sticky 信息轨、滚动与断点策略
- 让列表默认按最近活跃时间与价值信号排序
- 把 lint 门禁收绿

### Phase 2：治理能力拉开差距

- 引入“删除建议引擎”
- 输出结构化 Markdown 摘要、关键工件、待办和风险结论
- 增加隔离区浏览器、恢复入口、批量清理策略
- 增加配置修改、写回、回滚与审批提示

### Phase 3：生态吸收与兼容

- 扩展到 Gemini CLI、Aider、Cursor agent、Cline、Roo Code 等
- 兼容更多 transcript schema，并为每个适配器建立版本探针
- 引入插件式 adapter SDK，让社区可增量支持更多助手

### Phase 4：形成不可替代的 OSM 护城河

- 会话价值评分与保留建议模型
- 项目级知识蒸馏和周期性归档
- 可审计的敏感配置变更历史
- 让 OSM 成为“本地 AI coding agent 治理总控台”，而不是浏览器

## 7. 对外发布页的致谢策略

发布页建议增加单独的 `Acknowledgements / 致谢` 区块，明确写出：

- 哪些项目给了 OSM 重要设计启发
- 哪些项目的代码或协议被直接吸收
- 对应许可证和仓库链接
- OSM 在其基础上新增了哪些治理能力

这样既合规，也能把“生态孵化贡献”公开说清楚。

## 8. 已落地的竞品吸收流程

这轮缺口已经补上，但补的是“工程化 intake 流程”，不是简单把第三方代码直接塞进主仓。

当前已经落地：

- `third_party/upstreams/catalog.json`
  - 结构化记录仓库、许可证、吸收姿态、限制条件和候选模块
- `scripts/intake-upstreams.mjs`
  - 从 catalog 生成研究索引、单仓库研究卡片、发布致谢与镜像规划
- `third_party/upstreams/intake-manifest.json`
  - 为每个上游仓库生成稳定镜像目录，方便后续本地克隆与代码级比对
- `docs/release/open-source-attribution.md`
  - 统一收口发布页的感谢和 reference-only 说明

当前仍未完成：

- 还没有把任意一个 candidate-absorb 仓库做成 clean-room 级继承 PR
- 还没有建立自动 license 文本抓取与 SPDX 复核
- 还没有把镜像 clone/fetch 步骤纳入可联网验证脚本
