# F102: proxy-env 写入修复

**类型**: 修复 Feature（引用已有 spec/design，跳过 specify/design，保留 TDD + review + gate）

**问题**: `WslAdapter` 和 `LinuxAdapter` 的 `write_state` 对 `ID_PROXY_ENV` 直接返回 `Ok(())`，未实际写入代理环境变量到 `/etc/environment`。远程 adapter（F101）已正确实现，但本地单实例 adapter 仍为空操作。

**Authority Sources**:
- F002 spec: `features/002-wsl-support/spec.md` §FR-2.1.1
- F002 design: `features/002-wsl-support/design.md` §2.5
- 参考实现: `src-tauri/src/adapters/wsl_remote.rs:211-249`

**状态**: 实施完成，待 review + closeout
