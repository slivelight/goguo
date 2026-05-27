# F104: 续跑期间 UI 阻塞

**类型**: 修复 Feature（引用已有 spec/design，跳过 specify/design，保留 TDD + review + gate）

**问题**: 恢复操作期间缺少前端蒙层或后端状态锁，可能导致用户在恢复进行中触发其他网络修改操作，产生状态竞争。

**Authority Sources**:
- F001 spec: `features/001-baseline-restore/spec.md` §FR-2.6.2-R2

**状态**: 实施完成，待 review + closeout
