# Visual Regression Implementation Plan

日期：2026-03-23  
条目：`QLT-04`

## 目标

把 UI 回归从“手动肉眼检查”推进到“有脚本、有基线、有比较结果”的视觉验证链路。

## 设计

- `web/package.json` 增加 `e2e:visual`
- 新增独立 Playwright visual spec，锁总览页和会话详情页
- 使用 demo data 和 browser preview，保证截图稳定

## TDD

1. 先写 Node 红测，锁 visual script 和 visual spec 必须存在。
2. 补最小 visual spec 与脚本实现。
3. 用 `--update-snapshots` 生成基线，再回跑一次比较测试。
