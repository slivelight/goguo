# F101: 协同部署模式修复

**类型**: 修复 Feature（引用已有 spec/design，跳过 specify/design，保留 TDD + review + gate）

**问题**: F002 协同模式（Coordinated Mode）在 spec/ADR 中承诺双侧同时管理（Windows + WSL），代码实现使用 `cfg` 条件编译仅做了单侧适配，违反 ADR-0005 平台适配器模式。

**Authority Sources**:
- F002 spec: `features/002-wsl-support/spec.md` §FR-2.2.1-R4, SC-2
- F002 design: `features/002-wsl-support/design.md` §2.5
- ADR-0005: `docs/adr/0005-platform-adapter-pattern.md`
- ADR-0007: `docs/adr/0007-*.md`（新增，待完成）

**状态**: 实施完成，待 ADR-0007 + review
