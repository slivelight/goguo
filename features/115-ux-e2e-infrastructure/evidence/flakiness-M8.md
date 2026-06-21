# F115 M8 Flakiness Evidence（SC-7 验收）

> **来源**：`e2e/scripts/benchmark.sh --runs 10` × 2 次（spec FR-2.2.4 / SC-7）。
> **时机**：T-24（finalize 证据），10 连跑度量 × 2 独立样本。
> **执行环境**：WSL2 + tauri-driver v2.0.6 + WebKitWebDriver（apt `webkit2gtk-driver`）+ GoGuo release（sha 33f47e8）。
> **结论**：**SC-7 通过**——合计 **20/20 PASS，flakiness = 0%**，远低于 ≤10% 阈值。

---

## SC-7 阈值

| 指标 | 要求 | 实测（合计 20 次） | 结果 |
|------|------|------------------|------|
| flakiness（FAIL/total） | ≤ 10% | **0/20 = 0%** | ✅ PASS |
| PASS 计数 | — | 20 | — |
| FAIL 计数 | — | 0 | — |
| 每次测试用例数 | — | 8 passing（稳定） | — |

> SC-7 spec 原文：10 连跑中至少 9 次 PASS（flakiness ≤ 10%）。本证据含 **2 次独立 10 连跑**，均 10/10 全过。

---

## 样本 1（2026-06-21T03:34:12Z，sha 33f47e8）

| 序号 | 模式 | 耗时(s) | 结果 | 测试用例 |
|------|------|--------|------|---------|
| 1 | self-spawn | 32.60 | PASS | 8 passing |
| 2 | self-spawn | 30.04 | PASS | 8 passing |
| 3 | self-spawn | 28.51 | PASS | 8 passing |
| 4 | self-spawn | 30.28 | PASS | 8 passing |
| 5 | self-spawn | 32.10 | PASS | 8 passing |
| 6 | self-spawn | 28.13 | PASS | 8 passing |
| 7 | self-spawn | 30.34 | PASS | 8 passing |
| 8 | self-spawn | 25.07 | PASS | 8 passing |
| 9 | self-spawn | 28.82 | PASS | 8 passing |
| 10 | self-spawn | 29.42 | PASS | 8 passing |

**统计**：均值 **29.53s** / 标准差 **2.13s** / CV 7.21% / 范围 25.07~32.60s（极差 7.53s）

---

## 样本 2（2026-06-21T03:50:54Z，sha 33f47e8）

| 序号 | 模式 | 耗时(s) | 结果 | 测试用例 |
|------|------|--------|------|---------|
| 1 | self-spawn | 32.59 | PASS | 8 passing |
| 2 | self-spawn | 31.05 | PASS | 8 passing |
| 3 | self-spawn | 32.13 | PASS | 8 passing |
| 4 | self-spawn | 29.99 | PASS | 8 passing |
| 5 | self-spawn | 36.22 | PASS | 8 passing |
| 6 | self-spawn | 26.43 | PASS | 8 passing |
| 7 | self-spawn | 33.05 | PASS | 8 passing |
| 8 | self-spawn | 29.35 | PASS | 8 passing |
| 9 | self-spawn | 38.70 | PASS | 8 passing |
| 10 | self-spawn | 40.47 | PASS | 8 passing |

**统计**：均值 **33.00s** / 标准差 **4.33s** / CV 13.17% / 范围 26.43~40.47s（极差 14.04s）

---

## 合计 20 次分析

| 维度 | 样本 1（10 次）| 样本 2（10 次）| 合计（20 次）|
|------|---------------|---------------|-------------|
| 均值 | 29.53s | 33.00s | **31.27s** |
| 标准差 | 2.13s | 4.33s | 3.79s（跨样本池化）|
| CV | 7.21% | 13.17% | 12.11% |
| 最小 / 最大 | 25.07 / 32.60 | 26.43 / 40.47 | 25.07 / 40.47 |
| flakiness | 0% | 0% | **0%** |
| SC-2（≤70s）| PASS（40s 余量）| PASS（37s 余量）| PASS |

**两样本差异根因**：

1. **样本 1 紧接 mihomo 预热后**：cargo build / driver install 后立即跑，page cache 热；样本 2 在样本 1 后约 16 分钟跑，cache 部分冷却
2. **样本 2 含尾部抖动**：run 9/10（38.70s / 40.47s）拉高均值 + stddev；WSL2 + WebKitGTK 冷启动特性，非 spec 风险
3. **均 SC-2 PASS**：两样本均值差距 3.47s 在阈值 70s 的 5% 以内，无实质差异
4. **flakiness 一致 = 0%**：PASS/FAIL 维度完全稳定，耗时波动不转化为失败

---

## 与 M4（5 次均值）对比

| 维度 | M4（T-12，5 次）| M8 样本 1 | M8 样本 2 | M8 合计 |
|------|----------------|----------|----------|---------|
| 样本量 | 5 | 10 | 10 | **20** |
| 均值 | 28.95s | 29.53s | 33.00s | 31.27s |
| 标准差 | 2.40s | 2.13s | 4.33s | 3.79s |
| CV | 8.29% | 7.21% | 13.17% | 12.11% |
| flakiness | 0% | 0% | 0% | **0%** |
| SC-2 | PASS | PASS | PASS | PASS |

---

## 与 PoC（F114）对比

| 维度 | F114 PoC（5 次，pre-T-09）| F115 M8（合计 20 次，post-T-09）|
|------|-------------------------|-------------------------------|
| 均值 | ~95s | 31.27s |
| stddev | ~5s | 3.79s |
| flakiness | 0% | 0% |
| 改进幅度 | baseline | **-67%**（63.73s 下降）|

---

## 测试稳定性观察（20 次合计）

- ✅ **零失败**：20 次 run 中无 wdio 错误、无 timeout、无 IPC roundtrip 失败
- ✅ **零警告**：无 `"Tauri plugin not available"`（ADR-0008 已验证）
- ✅ **测试用例数稳定**：每次均 8 passing，20 次无 fluctuation
- ✅ **driver 生命周期无泄漏**：每次 run 后 tauri-driver + WebKitWebDriver 进程正常退出，无僵尸进程
- ⚠️ **耗时尾部效应**：样本 2 的 run 9/10 高于均值 ~17%~23%，但属 WSL2 + WebKitGTK 冷启动特性，非 spec 风险
- ✅ **跨样本一致性**：两次独立 10 连跑 flakiness 均 = 0%，证明稳定性非偶然

---

## References

- spec SC-7：`features/115-ux-e2e-infrastructure/spec.md` §1.4 + §3.2
- benchmark 脚本：`e2e/scripts/benchmark.sh`
- M4 Benchmark（含 2 次 10 连跑 summary）：同目录 `benchmark-M4.md` lines 38-68
- F114 PoC 5 次证据：`features/114-ui-e2e-poc/poc-report.md` §4 QG3
- ADR-0008（plugin 注册消除警告）：`docs/adr/0008-tauri-plugin-wdio-in-production-cargo-toml.md`
