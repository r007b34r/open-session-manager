# Mobile Viewport Implementation Plan

日期：2026-03-23  
条目：`UX-01`

## 目标

把现有 Web 端从“有窄视口样式”推进到“正式的 Playwright 多视口验证”。

## 设计

- Playwright 增加 `mobile-chrome` 项目，直接复用 `Pixel 7`
- 沿用已有浏览器 preview 启动链
- 复用当前窄视口 e2e，不新增与业务逻辑耦合的假覆盖

## TDD

1. 先写 Node 级红测，锁 `web/playwright.config.ts` 必须包含移动项目。
2. 最小实现把移动项目接进 Playwright 配置。
3. 用 `mobile-chrome` 真跑一条窄视口 e2e，确认不是静态声明。
