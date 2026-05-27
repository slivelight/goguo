# F103: 非目标站点可达性验证（SC-5）

**类型**: 修复 Feature（引用已有 spec/design，跳过 specify/design，保留 TDD + review + gate）

**问题**: `restore_to_baseline()` 完成后未验证非目标站点的可达性。spec SC-5 要求恢复后确认非目标站点不受影响。

**Authority Sources**:
- F001 spec: `features/001-baseline-restore/spec.md` §FR-2.4.3-R1, SC-5

**状态**: 实施完成，待 review + closeout
