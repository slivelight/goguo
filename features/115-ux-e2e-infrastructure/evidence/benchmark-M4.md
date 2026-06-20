# F115 M4 Benchmark Evidence

来源：`e2e/scripts/benchmark.sh`（FR-2.2.4 / SC-2）。

## 单次记录

| 序号 | 模式 | 耗时(s) | 结果 | 时间戳(UTC) | git sha |
|------|------|--------|------|------------|---------|
| 1 | self-spawn | 35.62 | PASS | 2026-06-20T07:36:37Z | 43ff421 |

### run summary @ 2026-06-20T07:36:37Z (sha 43ff421)
- 模式: self-spawn, 次数: 1
- 均值: **35.62s** | 标准差: 0.00s
- SC-2 阈值（仅自启模式）: PASS (≤ 70s)

| 1 | self-spawn | 30.76 | PASS | 2026-06-20T07:37:54Z | 43ff421 |
| 2 | self-spawn | 31.59 | PASS | 2026-06-20T07:37:54Z | 43ff421 |
| 3 | self-spawn | 29.43 | PASS | 2026-06-20T07:37:54Z | 43ff421 |
| 4 | self-spawn | 26.04 | PASS | 2026-06-20T07:37:54Z | 43ff421 |
| 5 | self-spawn | 26.91 | PASS | 2026-06-20T07:37:54Z | 43ff421 |

### run summary @ 2026-06-20T07:37:54Z (sha 43ff421)
- 模式: self-spawn, 次数: 5
- 均值: **28.95s** | 标准差: 2.40s
- SC-2 阈值（仅自启模式）: PASS (≤ 70s)

| 1 | reuse | 32.90 | PASS | 2026-06-20T07:40:46Z | 43ff421 |
| 2 | reuse | 33.22 | PASS | 2026-06-20T07:40:46Z | 43ff421 |
| 3 | reuse | 34.54 | PASS | 2026-06-20T07:40:46Z | 43ff421 |
| 4 | reuse | 52.30 | PASS | 2026-06-20T07:40:46Z | 43ff421 |
| 5 | reuse | 66.15 | PASS | 2026-06-20T07:40:46Z | 43ff421 |

### run summary @ 2026-06-20T07:40:46Z (sha 43ff421)
- 模式: reuse, 次数: 5
- 均值: **43.82s** | 标准差: 14.90s
- SC-2 阈值（仅自启模式）: N/A (reuse mode)

