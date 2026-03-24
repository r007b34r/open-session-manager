## 背景

`API-07` 还没有落地。当前 `serve` 只有 JSON 查询和控制接口，缺少能被 Prometheus 或本地监控直接采集的稳定文本指标。

## 方案

本次只补最小可用的本地 metrics 壳层：

- `GET /metrics`，输出 Prometheus text exposition
- 沿用现有 `serve` snapshot 数据，不新增独立采集线程
- 与现有受保护接口保持一致：配置了 Bearer token 时，`/metrics` 也要求鉴权
- OpenAPI 同步补 `/metrics` 文档，标明返回 `text/plain`

## 首批指标

- `osm_sessions_total`
- `osm_sessions_by_assistant{assistant="..."}`
- `osm_session_control_supported_total`
- `osm_session_control_available_total`
- `osm_configs_total`
- `osm_configs_by_assistant{assistant="..."}`
- `osm_git_projects_total`
- `osm_doctor_findings_total`
- `osm_audit_events_total`

## 测试入口

- `src-tauri/tests/http_api.rs`
  - `serve_command_exposes_prometheus_metrics`
  - `serve_command_requires_bearer_token_for_metrics_when_configured`

## 验收

- `/metrics` 返回 `200` 和 `text/plain`
- 指标文本包含会话总数、控制支持数、配置总数和审计事件总数
- 配置了 token 时，未授权请求返回 `401`
- OpenAPI 能看到 `/metrics` 路由
