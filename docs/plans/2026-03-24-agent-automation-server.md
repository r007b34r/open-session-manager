## 背景

`API-04` 还没有实现。OSM 已经有本地 HTTP 查询接口和控制接口，但缺一层适合脚本、agent、编排器调用的“任务提交 + 回执查询”模型。

## 方案

本次只补最小 automation server：

- `POST /api/v1/automation/tasks`
- `GET /api/v1/automation/tasks/{taskId}`

首批任务类型：

- `snapshot.refresh`
- `sessions.search`
- `sessions.resume`
- `sessions.continue`

任务先同步执行，但统一返回稳定 receipt，方便后续再扩成异步队列。

## 回执字段

- `taskId`
- `kind`
- `status`
- `submittedAt`
- `completedAt`
- `result`
- `error`

## 测试入口

- `src-tauri/tests/http_api.rs`
  - `serve_command_triggers_automation_search_task_and_returns_receipt`
  - `serve_command_triggers_automation_resume_task_and_returns_receipt`

## 验收

- 可以提交 automation task 并拿到 receipt
- 可以按 `taskId` 查询回执
- `sessions.search` 会返回命中结果
- `sessions.resume` 会复用现有控制链路并回传最新 session detail
- `sessions.continue` 会复用现有控制链路并回传最新 session detail
