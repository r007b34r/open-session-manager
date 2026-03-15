# 2026-03-16 会话交接摘要导出 Spec

日期：2026-03-16  
工作树：`feat/usability-clarity`

## 背景

OSM 现在已经能导出 Markdown，也能把 transcript highlights、Todo 快照和清理建议放进去，但它还缺一个更适合“删前保留核心价值”的交接层。现在的导出更像审计摘要，不像一份接手人可以直接继续推进的 brief。

`ChristopherA/claude_code_tools` 给出的启发不是“直接照搬它的技能文件”，而是它把会话收尾和恢复拆成了几个稳定字段：当前状态、下一步、未完成项、需要注意的同步点。这个思路和 OSM 的目标高度一致。

## 目标

在 OSM 的 Markdown 导出中增加一段稳定的“会话交接摘要”，让用户在删除或迁移原始会话前，先拿到一份更可继续使用的简报。

## 非目标

- 本轮不接真实会话恢复命令
- 不引入 LLM 二次总结
- 不改现有软删除前置条件
- 不做多语言 Markdown 模板切换

## 交付行为

导出的 Markdown 增加 `## Session Handoff` 段落，至少包含：

- `Next focus`：优先取第一个未完成 todo；没有 todo 时回退到 insight summary
- `Open tasks`：未完成 todo 数
- `Completed tasks`：已完成 todo 数
- `Resume cue`：优先取最新 assistant highlight；没有则回退到 summary

## 数据来源

- `TranscriptTodo`
- `TranscriptHighlight`
- `SessionInsight.summary`
- `SessionInsight.progress_state`

## 验收标准

- 含 Todo 的导出能稳定生成 handoff 段落
- 没有 Todo 的导出也能回退生成 handoff 段落
- 现有导出测试继续通过
- 新增/更新的测试先失败后通过
