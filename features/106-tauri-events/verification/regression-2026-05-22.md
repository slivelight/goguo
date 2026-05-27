# F106 Verification Record

- 日期：2026-05-22
- 类型：`hf-regression-gate`

与 F105 共享回归验证记录：`features/105-proxy-guard-background/verification/regression-2026-05-22.md`

## Event Coverage Verification

| 事件 | 状态 |
|------|------|
| `baseline:confirmed` | ✅ pre-existing |
| `baseline:deviation-detected` | ✅ pre-existing |
| `service:stopped` | ✅ pre-existing |
| `recovery:started` | ✅ F106 新增 |
| `recovery:completed` | ✅ F106 新增 |
| `recovery:failed` | ✅ F106 新增 |
| `service:started` | ✅ F105+F106 新增 |
| `proxy-guard:recovery-triggered` | ✅ F105+F106 新增 |
| `recovery:item-completed` | ❌ future work |

**覆盖率**：8/9 (89%)

## 结论

**PASS**
