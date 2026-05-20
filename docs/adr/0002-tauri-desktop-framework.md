# ADR-0002: Desktop App Framework — Tauri

- **Status**: accepted
- **Date**: 2026-05-12
- **Deciders**: 用户
- **Affected Features**: 001, 002, 003, 004

## Context

GoGuo 需要一个跨平台桌面应用，承载 Feature 001~004 定义的全部交互语义（Feature 004 CON-1）。应用需在 Windows 和 WSL/Linux 两侧运行，冷启动 <= 3s（Feature 004 NFR-3.1-1），全部数据来源于本地 API，不发起远程请求（Feature 004 CON-3）。

## Decision

**采用 Tauri 作为桌面应用框架。**

## Alternatives Considered

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **Tauri** | Rust 原生后端、小体积（~5MB）、系统 WebView、安全权限模型、跨 Windows/macOS/Linux | 需要 WebView2（Windows）/WebKitGTK（Linux）；前端框架需另行选择 | **选定** |
| Electron | 成熟生态、Node.js 后端、完整 Chromium | 大体积（~100MB+）、高内存、Chromium 安全面大 | 排除：体积与安全面不符合 CON-3 |
| Qt (Rust binding) | 原生控件、无 WebView 依赖 | Rust Qt binding 不成熟、UI 开发效率低、跨平台样式难一致 | 排除：生态不足 |
| Native (Win32 + GTK) | 最小依赖 | 双平台双 UI 代码库、维护成本高 | 排除：跨平台一致性成本过高 |

## Consequences

- **正面**: 与 Rust 后端语言统一（`coding-principles.md`）；冷启动满足 3s 目标；安全权限模型支持最小权限原则；二进制体积小，分发简单。
- **负面**: WSL/Linux 侧需安装 WebKitGTK 运行时依赖；Tauri 版本升级需同步 Rust + npm 包版本（`coding-principles.md` 已有 ABI 兼容性纪律）。
- **约束**: 前端框架选型独立为 ADR-0006；Tauri plugin 生态需评估是否覆盖系统通知需求（Feature 004 FR-2.7.2）。

## References

- `docs/principles/coding-principles.md`（Tauri 专项注意）
- `features/004-user-interaction/spec.md`（CON-1, CON-3, NFR-3.1-1）
