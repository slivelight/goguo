# ADR-0006: 前端框架选型 — React + TypeScript

- **Status**: accepted
- **Date**: 2026-05-12
- **Deciders**: 用户
- **Affected Features**: 004

## Context

Tauri 已确定为桌面框架（ADR-0002），需选择前端 UI 框架。约束：

1. 冷启动 <= 3s（Feature 004 NFR-3.1-1）：含 Tauri 启动 + WebView 初始化 + 首屏渲染
2. 跨平台 UI 一致性（Feature 004 CON-2）：Windows 和 WSL/Linux 两侧操作界面一致
3. 7 个 UI 模块、~40 FR：状态展示、站点管理、规则预览、诊断面板、通知、首次引导、服务控制
4. 不发起远程请求（Feature 004 CON-3）
5. 项目使用 pnpm 管理前端依赖（`AGENTS.md`）

## Decision

**采用 React + TypeScript + Vite 作为前端框架，搭配 shadcn/ui 组件库。**

## Alternatives Considered

| 方案 | 冷启动 | 生态 | Tauri 集成 | 组件库 | 结论 |
|------|--------|------|-----------|--------|------|
| **React + TypeScript** | 中（需 code splitting） | 最大 | 最佳（Tauri 官方示例均为 React） | shadcn/ui（轻量、可定制） | **选定** |
| Svelte + TypeScript | 优（编译到原生 JS，~20KB） | 中 | 良好 | 较少成熟选择 | 候选：生态不足以支撑 7 模块 |
| Solid + TypeScript | 优（最小运行时） | 小 | 有限 | 极少 | 排除：生态不足 |
| Vue + TypeScript | 中 | 大 | 良好 | Element Plus 等 | 排除：无显著优势 |

### 冷启动分析

Tauri + WebView 初始化约 500ms~1s（取决于平台），留给前端渲染的预算为 2~2.5s。

| 阶段 | React + code splitting | Svelte |
|------|----------------------|--------|
| Tauri 启动 + WebView | ~800ms | ~800ms |
| JS bundle 解析 + 执行 | ~300ms | ~100ms |
| React hydration / 首屏渲染 | ~200ms | ~50ms |
| API 调用 + 数据填充 | ~500ms | ~500ms |
| **总计** | **~1.8s** | **~1.45s** |

React + code splitting 满足 3s 目标，余量约 1.2s。

## Consequences

- **正面**: Tauri 官方 React 模板和插件生态最完善；shadcn/ui 提供高质量可定制组件；TypeScript 类型安全与 Rust 后端类型定义可共享（通过 ts-rs 等工具）；pnpm 原生支持。
- **负面**: React bundle 体积大于 Svelte/Solid（需 code splitting 控制）；VDOM 开销在低性能设备上可能更明显。
- **组件策略**: shadcn/ui 提供 Button、Dialog、Table、Toast 等基础组件；自定义业务组件按 Atomic Design 组织（Feature 004 各模块）。

## References

- `docs/adr/0002-tauri-desktop-framework.md`
- `features/004-user-interaction/spec.md`（NFR-3.1-1, CON-2, CON-3）
- `docs/principles/coding-principles.md`（Tauri 专项注意、pnpm）
