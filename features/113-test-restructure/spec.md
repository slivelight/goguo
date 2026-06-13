# Feature 113: 三层测试重构（F001~F004 历史补齐） — 需求规格

- **Feature**: 113-test-restructure
- **阶段**: `hf-specify`
- **状态**: 草稿
- **日期**: 2026-06-11
- **上游输入**:
  - `docs/principles/testing-principles.md` — 三层测试方法论（宪法层）
  - `features/001-baseline-restore/spec.md` ~ `features/004-user-interaction/spec.md` — FR 来源
  - F109、F110 spec + design — 已知 gap 清单（验收测试标签来源）
  - `docs/insights/2026-05-21-v010-spec-design-impl-drift-audit.md` — v0.1.0 审计报告

## 1. 概述

### 1.1 目的

本 feature 为 F001~F004 的 142 条 FR 建立三层自动化验收测试体系。核心目标：**每条 FR 有且仅有一个验收测试，断言 spec 描述的用户可观测结果**。

本 feature **只写测试，不修生产代码**。失败的测试标记 `#[ignore]` 并指向负责修复的 feature（F109/F110 或新开 F114+）。

### 1.2 范围

| 层级 | 覆盖范围 | 预估数量 |
|------|---------|---------|
| FR 验收测试 | F001~F004 共 142 条 FR | ~61 |
| 契约测试 | 适配器一致性、DTO 往返、trait 行为 | ~20 |
| 管道集成测试 | 7 条端到端数据流链路 | ~7 |
| FR 追溯矩阵 | 142 FR × 测试函数 × 状态 | 1 文档 |
| **合计** | | **~88** |

### 1.3 术语

| 术语 | 定义 |
|------|------|
| FR 验收测试 | 以 spec FR 为来源，断言用户可观测结果的自动化测试 |
| 契约测试 | 断言接口一致性的自动化测试 |
| 管道集成测试 | 断言端到端数据流连通的自动化测试 |
| 追溯矩阵 | FR ID → 测试函数名 → 状态的映射表 |
| 可观测结果 | 用户通过 UI 或系统状态可直接感知的变化 |
| 发现分流 | 失败测试按根因归入已有 fix feature 或新开 F114+ |

### 1.4 成功标准

| # | 标准 | 验证方式 |
|---|------|----------|
| SC-1 | 142 条 FR 每条有且仅有一个验收测试（通过或 `#[ignore]`） | 追溯矩阵 100% 填写 |
| SC-2 | 每个 `#[ignore]` 测试指向一个具体的修复 feature ID | `grep '#\[ignore'` 输出含 feature 编号 |
| SC-3 | 所有契约测试通过（适配器分类一致、DTO 往返正确） | `cargo test contract_` 全绿 |
| SC-4 | 所有管道集成测试通过（7 条端到端链路连通） | `cargo test pipeline_` 全绿 |
| SC-5 | 测试基建（setup helper）可复用，新增验收测试无需手写 mock | helper 函数覆盖 4 种标准状态 |
| SC-6 | CI 集成：`cargo test fr_` 作为 PR 合并门 | CI 配置已更新 |

## 2. 功能需求

### 2.1 FR 追溯矩阵

#### FR-2.1.1 追溯矩阵建立

**要求**:
- FR-2.1.1-R1: 必须建立 `docs/test-trace-matrix.md`，包含 142 条 FR 的完整映射
  - 验收测试: `fr_2_1_1_trace_matrix_complete`
  - 可观测结果: 追溯矩阵中每条 FR 对应的测试函数名非空
- FR-2.1.1-R2: 追溯矩阵每条记录必须包含：FR ID、测试函数名、状态（通过/忽略/缺失）、修复 feature ID（如适用）
- FR-2.1.1-R3: 追溯矩阵必须标注 FR 分组关系（多条 FR 共享同一验收测试时标注）

### 2.2 FR 验收测试（后端）

#### FR-2.2.1 F001 baseline-restore 验收测试

**要求**:
- FR-2.2.1-R1: F001 的 52 条 FR 必须有对应的验收测试，放置在 `src-tauri/tests/fr_acceptance/f001_baseline.rs`
  - 验收测试: 文件存在且 `cargo test --test fr_acceptance f001` 可执行
  - 可观测结果: 测试文件包含 `fr_` 前缀的测试函数，覆盖 F001 spec §2 全部 FR
- FR-2.2.1-R2: 验收测试必须使用 setup helper 构建测试状态，不直接构造 mock
- FR-2.2.1-R3: 已知 gap 对应的验收测试必须标记 `#[ignore = "F109/F110: gap 描述"]`

#### FR-2.2.2 F002 wsl-support 验收测试

**要求**:
- FR-2.2.2-R1: F002 的 19 条 FR 必须有对应的验收测试，放置在 `src-tauri/tests/fr_acceptance/f002_wsl.rs`
- FR-2.2.2-R2: WSL 特定测试在非 WSL 环境标记 `#[ignore]`，注释说明原因

#### FR-2.2.3 F003 site-rules 验收测试

**要求**:
- FR-2.2.3-R1: F003 的 28 条 FR 必须有对应的验收测试，放置在 `src-tauri/tests/fr_acceptance/f003_site_rules.rs`

#### FR-2.2.4 F004 user-interaction 后端验收测试

**要求**:
- FR-2.2.4-R1: F004 的后端相关 FR（Tauri command 行为）必须有对应的验收测试，放置在 `src-tauri/tests/fr_acceptance/f004_backend.rs`

### 2.3 FR 验收测试（前端）

#### FR-2.3.1 F004 前端验收测试

**要求**:
- FR-2.3.1-R1: F004 的前端相关 FR（UI 渲染、交互行为）必须有对应的验收测试，放置在 `src/__tests__/fr-acceptance/f004-ui.test.tsx`
  - 验收测试: 文件存在且 `pnpm test -- fr_` 可执行
  - 可观测结果: 测试断言 DOM 渲染内容（文本、元素存在性），不断言 store 内部状态
- FR-2.3.1-R2: 前端验收测试必须验证 spec 描述的用户可感知文本和交互元素

### 2.4 契约测试

#### FR-2.4.1 适配器一致性契约

**要求**:
- FR-2.4.1-R1: 所有适配器（Windows/Linux/WSL/WslRemote）对同一 state_item_id 的 category 必须一致
  - 验收测试: `contract_adapter_category_consistent`
  - 可观测结果: 无不一致的 category 被发现
- FR-2.4.1-R2: 所有适配器的 `state_item_definitions()` 返回结构必须包含相同的必需字段

#### FR-2.4.2 Tauri command DTO 契约

**要求**:
- FR-2.4.2-R1: 所有 Tauri command 的 Request/Response 类型必须通过序列化往返测试（roundtrip）
- FR-2.4.2-R2: 新增的 Response 类型（如 `RestoreBaselineResponse`、`ApplyRulesResponse`）必须有独立的 DTO 契约测试

#### FR-2.4.3 Trait 行为契约

**要求**:
- FR-2.4.3-R1: `ProbeClient` trait 的所有实现（Mock + Real）必须通过共享的行为契约测试套件
- FR-2.4.3-R2: `PlatformAdapter` trait 的所有实现必须通过共享的行为契约测试套件（含新增的 `read_state`、`get_wsl_network_info`）

### 2.5 管道集成测试

#### FR-2.5.1 核心管道验证

**要求**:
- FR-2.5.1-R1: 以下 7 条端到端链路必须有独立的管道集成测试：
  1. 评估 → 确认 → 恢复 → 审计（F001 全链路）
  2. 订阅导入 → NodePool → 规则生成 → mihomo 重载（F003 链路）
  3. 站点添加 → 规则生成 → 探测 → 可达性展示（F003+F004 链路）
  4. 探测 → 五要素诊断 → 审计（F001+F003 链路）
  5. ProxyGuard → 系统代理清除 → 恢复（F001 链路）
  6. Wizard → 调整建议 → 重新采集（F004 链路）
  7. 规则生成 → 预览 → 应用 → 非目标验证 → 回退（F003 链路）
  - 验收测试: 7 个 `pipeline_*` 测试存在且可执行
  - 可观测结果: 每条管道终点的状态符合 spec 描述

### 2.6 测试基建

#### FR-2.6.1 Setup Helper 库

**要求**:
- FR-2.6.1-R1: 必须提供统一的测试状态构建函数，位于 `src-tauri/tests/common/mod.rs`：
  - `setup_baseline_confirmed()` — baseline 已确认状态
  - `setup_service_running()` — mihomo 运行中状态
  - `setup_service_stopped()` — mihomo 已停止状态
  - `setup_site_rules_with_nodes()` — 站点规则 + 节点池已填充状态
  - 验收测试: `fr_2_6_1_r1_setup_helpers_exist`
  - 可观测结果: 4 个 helper 函数存在且返回有效 AppState/SiteRulesState
- FR-2.6.1-R2: Helper 函数内部使用 mock 适配器，不依赖真实系统状态
- FR-2.6.1-R3: Helper 函数的 mock 行为必须与 spec 描述一致（如 mock adapter 的 write_state 记录调用）

### 2.7 发现分流

#### FR-2.7.1 新发现问题处理

**要求**:
- FR-2.7.1-R1: 编写验收测试时发现的每项新失败必须记录到 F113 progress.md
- FR-2.7.1-R2: 分流规则按 `testing-principles.md` §4 执行：
  - 与 F109/F110 gap 同根 → 追加到该 feature
  - 独立 P0/P1 → 新开 F114+
  - 独立 P2 → 累积到批次
- FR-2.7.1-R3: 分流结果必须在追溯矩阵中标注

## 3. 非功能需求

### 3.1 性能

| ID | 需求 | 验证方式 |
|----|------|----------|
| NFR-3.1-1 | 验收测试套件全量执行 ≤ 60s | CI 计时 |
| NFR-3.1-2 | 单个验收测试执行 ≤ 5s | `cargo test -- --nocapture` 计时 |

### 3.2 可维护性

| ID | 需求 | 验证方式 |
|----|------|----------|
| NFR-3.2-1 | 新增验收测试无需修改 setup helper | 在已有 helper 上编写 3 个新测试验证 |
| NFR-3.2-2 | FR 追溯矩阵可通过脚本验证覆盖率 | `python3 scripts/verify-fr-coverage.py` |

### 3.3 兼容性

| ID | 需求 | 验证方式 |
|----|------|----------|
| NFR-3.3-1 | 新测试不得破坏现有 923 个测试 | `cargo test` 全量通过 |
| NFR-3.3-2 | 前端验收测试不得引入新的 npm 依赖 | `package.json` 无新增 |

## 4. 约束

| ID | 约束 | 原因 |
|----|------|------|
| CON-1 | 不修改任何生产代码（仅新增测试文件和测试基建） | F113 是探测器不是手术刀 |
| CON-2 | 不修改 F001~F004 已关闭的 spec/design 文档 | 历史锚点不可变 |
| CON-3 | 验收测试断言可观测结果，不断言函数返回值或内部状态 | testing-principles.md §1 |
| CON-4 | 失败测试必须标记 `#[ignore]`，不得提交 RED 状态的测试 | CI 稳定性 |
| CON-5 | 不引入新的 crate 依赖 | 测试基建复用现有 mock |

## 5. 不在范围内

| 排除项 | 原因 |
|--------|------|
| 修复任何生产 bug | F113 只发现，不修复 |
| NFR 自动化测试（性能计时、安全审计） | 大多需手动验证，ROI 不够 |
| E2E 跨前后端测试（Playwright） | 需评估 ROI，不在本 feature 范围 |
| 未来 feature 的验收测试 | 未来 feature 在自身 hf-test-driven-dev 中编写 |
| 修改 F109/F110 的 `#[ignore]` 标记 | 由 F109/F110 修复时自行移除 |

## 6. 假设

| ID | 假设 | 置信度 | 验证方式 |
|----|------|--------|----------|
| ASM-1 | 现有 mock adapter 可被复用为验收测试的状态构建基础 | 高 | setup helper 实现 |
| ASM-2 | F109/F110 修复后的 API 接口与当前 spec 描述一致 | 高 | F109/F110 spec 为 authority |
| ASM-3 | 142 条 FR 可通过分组压缩到 ~61 个验收测试 | 高 | 追溯矩阵建立时验证 |

## 7. 开放问题

| ID | 问题 | 阻塞性 | 建议处理时机 |
|----|------|--------|-------------|
| OP-1 | 前端验收测试是否需要 Playwright，还是 vitest 足够 | 非阻塞 | hf-design 阶段决定 |
| OP-2 | `verify-fr-coverage.py` 脚本是否需要开发，还是追溯矩阵人工维护即可 | 非阻塞 | hf-design 阶段决定 |
| OP-3 | WSL 特定测试在 CI（GitHub Actions Linux runner）中如何执行 | 非阻塞 | hf-design 阶段 |
