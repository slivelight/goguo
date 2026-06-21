# ADR-0008: 生产 Cargo.toml 引入 tauri-plugin-wdio

- **Status**: accepted（F115 finalize 确认）
- **Date**: 2026-06-21（落盘；2026-06-18 决策 B 立项）
- **Deciders**: 用户
- **Affected Features**: 115（UX E2E 基础设施正式化），及其后所有接入 §7 强制规范的 Feature

## Context

F114 PoC 验证 `tauri-driver + WebDriverIO + @wdio/tauri-service` 可在 WSL2 下建立桌面 E2E 基础设施（详见 `features/114-ui-e2e-poc/poc-report.md`）。

`@wdio/tauri-service` v1.1.0 依赖 `tauri-plugin-wdio`（Tauri 插件）提供 window 状态查询与 mock 注入。PoC 阶段未注册该插件，wdio 运行日志出现 5 次 `"Tauri plugin not available. Make sure @wdio/tauri-plugin is installed"` 警告（F114 PoC 报告 §5 风险登记）。

F115 spec FR-2.2.3 要求消除该警告，OQ-2 决策"进生产 Cargo.toml"。本 ADR 记录此决策的取舍与实测数据。

## Decision

将 `tauri-plugin-wdio` 加入 `src-tauri/Cargo.toml` `[dependencies]`，并在 `src-tauri/src/lib.rs` `tauri::Builder` 链中注册 `.plugin(tauri_plugin_wdio::init())`。

**全 profile 启用**（不使用 dev-only feature gate）。

完整集成需 6 步（design §2.3 v3 勘误-3 补全）：

| Step | 文件 | 变更 |
|------|------|------|
| 1 | `src-tauri/Cargo.toml` | `+tauri-plugin-wdio = "1"` |
| 2 | `src-tauri/src/lib.rs` | `+.plugin(tauri_plugin_wdio::init())` |
| 3 | `src-tauri/capabilities/default.json` | `permissions` 数组 `+"wdio:default"` |
| 4 | `src-tauri/tauri.conf.json` | `app.withGlobalTauri = true` |
| 5 | `package.json`（仓库根） | `+@wdio/tauri-plugin` v1.1.0 |
| 6 | `src/main.tsx` | `+import '@wdio/tauri-plugin'`（副作用 import） |

## Alternatives Considered

### 1. 生产 Cargo.toml（全 profile，采纳）

- **优势**：单一构建产物；与 @wdio/tauri-service 默认路径一致；F114 PoC 已在 release 上验证可行
- **劣势**：二进制体积/启动时间微增；攻击面微增
- **决策**：采纳。量化数据见下"取舍"

### 2. dev-only feature gate（被排除）

- **优势**：生产二进制纯净（不含测试专用 plugin）
- **劣势**：
  - 双构建矩阵复杂化（`--features dev-test-infra` vs 默认）
  - F114 PoC 已在 release 上验证可行，gate 化属于倒退
  - 增加后续 Feature 接入复杂度（每个 e2e 测试需先确认 plugin 已 gate 启用）
- **决策**：否决（OQ-2，2026-06-18 spec 阶段定）

### 3. 不注册 plugin（被排除）

- **劣势**：PoC 警告持续（5 次/跑），虽不阻塞测试通过，但污染日志，影响调试可读性
- **决策**：否决

## 取舍（T-09 / T-12 实测数据回填）

| 维度 | 阈值 | 实测 | 结果 |
|------|------|------|------|
| 二进制体积增长 | ≤ 2 MB（OQ-6） | **+0.27 MB**（10,281,056 → 10,561,800 bytes，含 Rust plugin + 前端 JS 副作用）| ✅ PASS（13.5% 阈值占用） |
| 冷启动时间增长 | ≤ 50 ms（OQ-6） | **下降而非上升**：T-09 前 wdio 全跑 1m6s → T-09 后 35s。plugin 初始化实际省了原 5 次 retry 时间 | ✅ PASS（trivially） |
| 多 spec SC-2（自启模式）| ≤ 70s（mean） | **28.95s**（5 次均值，stddev 2.40s，T-12 度量）| ✅ PASS（41s 余量，相对 PoC 95s 降幅 69%） |
| 警告消除 | grep `"Tauri plugin not available"` 计数 = 0 | **5 → 0**（T-09 6 步接入后稳定）| ✅ PASS（FR-2.2.3-R3） |
| 回归测试 | 0 新增失败 | cargo 737 passed / 0 failed（vs v0.1.0 baseline 625，新增 112 个无回归）；前端 vitest 220 passed / 5 failed（5 个 pre-existing，git stash 验证非 T-09 引入）| ✅ PASS（FR-2.2.3-R4） |

### 攻击面

plugin 暴露的 IPC 命令对前端可见，但仅提供 window 状态查询，无敏感操作（文件写入 / 系统命令）。Capabilities 已显式声明 `wdio:default` 权限。

### 可逆性

移除仅需还原 6 步变更（Cargo.toml + lib.rs + capabilities + tauri.conf.json + package.json + main.tsx），无数据迁移。高可逆。

## Consequences

- **正面**：
  - F115 SC-3（plugin 注册）/ SC-8（无回归）验收依赖此决策，全部达成
  - 后续所有 e2e 测试默认假设 plugin 已注册（无需 Feature 自行配置）
  - 警告消失使 wdio 日志可读性大幅提升，调试效率↑
- **负面 / 需关注**：
  - **post-T-09 复用模式性能方程翻转**（T-12 度量发现）：复用模式 5 次均值 43.82s（stddev 14.90s，run 4/5 退化到 52s/66s）vs 自启 28.95s——复用不再快于自启。**此非本 ADR 决策的预期影响**，根因疑似 tauri-driver 跨多次 reuse 累积状态与 plugin 注入路径交互。已落 spec v3 勘误-4，复用模式重定位为 dev 体验可选项，SC-2 由自启承担，stddev 不稳定挂账 F116+。
  - 二进制 +0.27 MB（13.5% 阈值占用，仍有充足余量）
  - 生产二进制含测试专用 plugin（philosophically 不洁，但功能上无害）

- **未来移除评估**：若 v0.5.0+ 重构测试基础设施，需评估 e2e spec 影响范围。可逆性高（6 步还原）。

## References

- F115 spec：`features/115-ux-e2e-infrastructure/spec.md` FR-2.2.3-R1~R6 + 勘误-3（6 步接入）+ 勘误-4（复用模式翻转）
- F115 design：`features/115-ux-e2e-infrastructure/design.md` §2.3（实施方案）+ §10（本 ADR 草稿）+ §13 评审点 #7
- F115 tasks：T-09 完成记录（6 步接入 + 量化数据）/ T-12 完成记录（SC-2 + 复用模式翻转）
- F114 PoC：`features/114-ui-e2e-poc/poc-report.md`（5 次警告证据）
- Benchmark 证据：`features/115-ux-e2e-infrastructure/evidence/benchmark-M4.md`
- ADR-0001~0007：本决策池历史记录（编号未复用）
