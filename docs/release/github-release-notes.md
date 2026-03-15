# open Session Manager v0.2.0 Public Preview

## 发布定位

`open Session Manager`（`OSM`）是一个面向 Win11、Linux 和 WSL 用户的本地优先治理平台，用来统一识别终端代码助手的会话、配置和高风险密钥引用，并在真正删除之前先帮助用户判断哪些内容值得保留、迁移或提炼。

本次 `v0.2.0` 不是“跨平台安装包全部完备”的稳定版，而是首个公开预览版：

- Windows 11 已具备桌面调试构建、真实本地快照、导出、软删除、恢复和 E2E 验证证据
- Linux / WSL 当前以发现、解析、审计和路径模型能力预览为主
- 对外发布的重点是把核心治理能力、公开文档、验证脚本和开源吸收边界完整公开

## 当前已吸收的能力

本版已经完成两条真实上游吸收链路：

- `daaain/claude-code-log`
  - 已吸收：更丰富的 Markdown 导出分节、transcript highlights、Claude todo snapshot
  - 对应到 OSM：删除前先形成更有价值的 Markdown 资产，而不是只做原始 transcript 归档
- `d-kimuson/claude-code-viewer`
  - 已吸收：viewer 风格 transcript detail 面板、session todo evidence 展示、Claude todo 提取思路
  - 对应到 OSM：会话详情更接近真实工作痕迹，而不是只显示元数据列表

本版同时公开了 upstream intake pipeline，把研究、许可证姿态、镜像规划和发布致谢收口成可复用工程资产：

- `third_party/upstreams/catalog.json`
- `third_party/upstreams/intake-manifest.json`
- `docs/research/upstreams/index.md`
- `docs/release/open-source-attribution.md`
- `scripts/intake-upstreams.mjs`

## 当前已实现的能力

- 支持 `Codex`、`Claude Code`、`OpenCode` 三类适配器
- 支持 Win11、Linux、WSL 的本地路径发现与真实 snapshot 管线
- 支持会话主题、摘要、进度、价值分、风险标记、最后活跃时间与 transcript digest
- 支持配置审计、密钥脱敏、第三方中转与危险权限风险识别
- 支持 Markdown 导出、软删除、恢复、持久化审计历史
- 支持“先导出 Markdown，再允许软删除”的前后端双重守卫
- 支持中英文切换，并按系统 / 浏览器语言自动选择默认语言
- 支持桌面 GUI 启动，Windows release `exe` 不再额外弹出终端窗口
- 支持更稳定的 Sessions 选中逻辑、搜索筛选回退和响应式详情布局
- 修复恢复越界、OpenCode 假删除和 Claude `TodoWrite` 漏提取问题

## 本版重点更新

- 增加 Tauri 桌面运行时，桌面端可直接调用 Rust 原生命令读取真实本地 snapshot
- 增加真实 snapshot 管线与持久化审计历史读取
- 增加中英文切换，并按系统 / 浏览器语言自动选择默认语言
- 增加“导出 Markdown 之后才允许软删除”的前后端双重守卫
- 增加统一验证入口，可一次性运行 Rust、Web、桌面构建和 E2E
- 增加 upstream intake pipeline，把竞品研究、许可证姿态、镜像规划和发布致谢收口成结构化产物
- 修复 Windows release `exe` 弹出终端窗口的问题
- 重做 Sessions 工作区的选中逻辑、筛选回退和响应式详情布局
- 收紧导出 / 隔离区路径生成，避免异常 `session_id` 造成路径越界
- 将桌面快照、导出和软删除命令改成异步执行，降低 UI 卡死风险
- 为 restore 增加受管隔离区和允许恢复根边界校验
- 为 Markdown frontmatter 增加安全转义，避免不规范 YAML 导出

## 验证结果

本版发布前已经通过以下本地验证：

- `cargo test --lib`
- `cargo test --test cli_snapshot`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `npm --prefix web run test`
- `npm --prefix web run build`
- `npm --prefix web run e2e`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

## 快速开始

```bash
npm --prefix web install
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
npm --prefix web run tauri:dev
```

## 构建与产物

```bash
npm --prefix web run tauri:build
```

当前 Windows 调试构建产物：

```text
target/debug/open-session-manager-core.exe
```

当前 Windows release 构建产物：

```text
target/release/open-session-manager-core.exe
```

## 当前边界

以下内容不包含在 `v0.2.0 Public Preview` 承诺范围内：

- Linux 桌面实机构建与真实目录回归证据闭环
- 完整的隔离区浏览器与恢复 UI
- 配置修改 / 删除后写回真实文件 UI
- MSI / AppImage / deb / 签名产物链路
- WSL companion collector 闭环

## 发布后优先项

以下工作按当前计划在本次发布之后继续推进：

- 真实 Win11 + WSL 多发行版样本回放
- 符号链接 / 目录联接逃逸专项测试
- 超大历史库性能压测
- 发布包 smoke test

## 开源致谢

以下项目对 OSM 的孵化有直接帮助：

- `jazzyalex/agent-sessions`
- `lulu-sk/CodexFlow`
- `d-kimuson/claude-code-viewer`
- `daaain/claude-code-log`
- `Dimension-AI-Technologies/Entropic`
- `yoavf/ai-sessions-mcp`
- `Dicklesworthstone/coding_agent_session_search`

当前版本已经把开源来源说明、许可证姿态和 reference-only 边界收口到：

- `docs/release/open-source-attribution.md`
- `docs/research/upstreams/index.md`
- `third_party/upstreams/catalog.json`

其中 MIT / Apache-2.0 等兼容许可证项目，会随着后续真正的代码级继承继续补充更细的来源说明；带附加限制的仓库仅吸收思路，不直接复制代码。
