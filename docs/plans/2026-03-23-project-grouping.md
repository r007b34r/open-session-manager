# Project Grouping Implementation Plan

日期：2026-03-23  
条目：`GIT-05`

## 目标

把 Sessions 和 Configs 从“平铺列表”推进到“按项目聚合”的视图，至少满足：

- Sessions 页面按 `projectPath` 分组
- Config 页面按推导出的项目根路径分组
- 分组后的交互不影响现有搜索、筛选、选择和配置编辑

## 设计

- `SessionTable` 按 `projectPath` 生成分组 section
- `ConfigRiskPanel` 按常见项目级配置路径后缀推导项目根并分组
- 继续沿用原有卡片和表格，不引入新的数据源

## TDD

1. 先写 SessionsRoute 测试，锁同一项目下的多个会话必须进入同一组。
2. 再写 ConfigRiskPanel 测试，锁同一项目根下的多份配置必须进入同一组。
3. 通过 `app.test.tsx` 和现有配置编辑测试确认分组没有破坏原交互。
