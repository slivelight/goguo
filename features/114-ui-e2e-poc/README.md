# F114 — UI E2E 自动化 PoC

| 字段 | 值 |
|------|----|
| 类型 | 基础设施 PoC（Spike） |
| 状态 | ✅ 已关闭（2026-06-18） |
| Authority Source | `poc-report.md`（PoC 阶段，无 spec/design） |
| 关联 | F111（WSL2 freeze，环境教训来源） |

## 一句话总结

WSL2 headless 下基于 **tauri-driver + WebDriverIO + @wdio/tauri-service** 的桌面 E2E 可行性已验证，三个 Quality Gate 全部通过（IPC roundtrip 实测、5/5 flakiness=0%）。建议立项 F115 做正式 Feature 覆盖。

详见 [poc-report.md](./poc-report.md)。
