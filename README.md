# open Session Manager

## Web 启动

```bash
npm --prefix web install
npm --prefix web run browser
```

打开 `http://127.0.0.1:4173`。

这个入口用于浏览器预览界面和交互。要读取本机真实会话、配置和控制能力，直接运行仓库内的桌面可执行文件。

## 当前版本已实现

- 支持识别 `Codex`、`Claude Code`、`OpenCode`、`Gemini CLI`、`GitHub Copilot CLI`、`Qwen CLI`、`Roo Code`、`Factory Droid`、`OpenClaw` 的本地会话
- 支持展示会话主题、摘要、进度、风险、usage/cost、MCP 服务和审计记录
- 支持搜索、Markdown 导出、cleanup checklist、软删除、恢复
- 支持 `GitHub Copilot CLI`、`Gemini CLI`、`Qwen CLI`、`Roo Code`、`Factory Droid`、`OpenClaw` 的配置审计与可视化修改，并带备份和回滚
- 支持中英文切换、浅色/深色主题，以及桌面端真实本地快照
