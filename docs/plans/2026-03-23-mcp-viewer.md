# MCP Viewer Implementation Plan

日期：2026-03-23  
条目：`MCP-02`

## 目标

把已经存在于配置审计链路里的 `mcp_json` 真正暴露到产品界面里，让用户能看到：

- 当前本地配置里有哪些 MCP server
- 每个 server 的状态、传输方式、命令或 URL
- 原始配置片段

## 设计

- Rust 侧在 `ConfigRiskRecord` 里新增 `mcpServers`
- 从各助手配置的 `mcp_json` / `mcpServers` / `servers` 结构派生统一记录
- Web 侧在 Config 页面增加 MCP viewer，不碰写回链路

## TDD

1. 先写 Rust 集成测试，锁 fixture snapshot 必须暴露 `filesystem/postgres` 等 MCP server。
2. 再写 Config 页面测试，锁 viewer 必须显示状态、传输方式、命令和原始配置。
3. 最小实现转绿后，补 dashboard 与 app 相关回归。
