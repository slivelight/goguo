# 三层测试方法论

> 本文档为项目宪法层文档，所有 feature 的测试活动必须遵循。
> 确立日期：2026-06-11

## 1. 核心原则

**测试验证 spec 被满足，而非代码能运行。**

测试的来源是 spec 的 FR（功能需求），不是实现代码。断言的对象是用户可观测的结果，不是函数的返回值。

## 2. 三层结构

### 2.1 第一层：FR 验收测试

- **验证目标**：spec 的每条 FR 是否被正确实现
- **断言对象**：用户可观测的系统状态变化（如 mihomo 进程是否存在、系统代理是否清除、审计日志是否包含特定记录）
- **编写时机**：hf-test-driven-dev 阶段，RED 起点是 spec FR，不是代码规划
- **命名规范**：`fr_<section>_<requirement>_<short_description>`
  - 例：`fr_2_1_1_r1_restore_stops_mihomo_and_clears_proxy`
- **目录结构**：
  - 后端：`src-tauri/tests/fr_acceptance/<feature>.rs`
  - 前端：`src/__tests__/fr-acceptance/<feature>.test.tsx`

### 2.2 第二层：契约测试

- **验证目标**：接口一致性（同一 trait 的不同实现行为一致、DTO 序列化正确、适配器分类统一）
- **断言对象**：接口契约（如同一 state_item_id 在所有适配器中 category 一致）
- **编写时机**：hf-test-driven-dev 阶段，与 FR 验收测试同步
- **命名规范**：`contract_<interface>_<behavior>`
  - 例：`contract_adapter_proxy_env_category_consistent`
- **目录结构**：同 FR 验收测试目录

### 2.3 第三层：管道集成测试

- **验证目标**：端到端链路连通（从输入到最终输出的完整数据流）
- **断言对象**：管道终点的状态（如 import_subscription 后 NodePool 非空）
- **编写时机**：hf-test-driven-dev 阶段，覆盖跨模块数据流
- **命名规范**：`pipeline_<source>_to_<sink>`
  - 例：`pipeline_subscription_to_node_pool`
- **目录结构**：`src-tauri/tests/integration_*.rs`（已有惯例）

## 3. HF 阶段产出物扩展

### 3.1 hf-specify

spec 中每条 FR 增加验收测试字段：

```markdown
- FR-2.1.1-R1: 采集必须在修改任何网络配置之前完成
  - 验收测试: `fr_2_1_1_snapshot_before_modification`
  - 可观测结果: 快照时间戳早于任何 write_state 调用
```

### 3.2 hf-tasks

每个 task 的 DoD 增加两条：

```
- [ ] 本 task 覆盖的 FR 验收测试已编写
- [ ] 验收测试断言的是 spec 描述的用户可观测结果，而非函数返回值
```

### 3.3 hf-test-driven-dev

RED-GREEN-REFACTOR 循环的 RED 起点：

```
原：读代码规划 → 写测试 → 实现
新：读 spec FR → 写验收测试 → 实现 → 验收测试通过
```

### 3.4 hf-finalize

关闭门增加一条：

```
- [ ] spec 中每条 FR 都有对应的通过的验收测试
  （可运行 `cargo test fr_` / `pnpm test -- fr` 验证）
```

## 4. 测试发现问题的处理规则

1. **测试重构 feature 只写测试，不修代码**
2. 失败测试标记 `#[ignore]`，注释指向修复 feature ID
3. 新发现问题的分流：

| 条件 | 去向 |
|------|------|
| 与已有 fix feature gap 同根（同模块、同根因） | 追加到该 fix feature |
| 全新问题，影响核心价值（P0/P1） | 新开 fix feature（F114+） |
| 全新问题，仅影响体验（P2） | 累积到批次处理 |

4. 修复 feature 的 DoD 包含：移除对应的 `#[ignore]` 标记
5. 未来 feature 的测试在 hf-test-driven-dev 阶段同步编写，发现的问题即时记录到本 feature 的 bug 列表

## 5. FR 追溯矩阵

- **格式**：FR ID | 测试函数名 | 状态（通过/忽略/缺失）| 修复 feature（如适用）
- **位置**：`docs/test-trace-matrix.md`
- **更新时机**：每个 feature closeout 时更新
- **目标**：142 条 FR（F001:52 + F002:19 + F003:28 + F004:43）每条有且仅有一个验收测试

## 6. 测试基建

### 6.1 后端 setup helper

统一的测试状态构建函数，位于 `src-tauri/tests/common/mod.rs`：

```rust
pub fn setup_baseline_confirmed() -> AppState { ... }
pub fn setup_service_running() -> AppState { ... }
pub fn setup_site_rules_with_nodes() -> SiteRulesState { ... }
```

### 6.2 CI 集成

```bash
# 验收测试套件（含忽略的测试信息）
cargo test fr_ -- --include-ignored 2>&1 | tee fr-acceptance-report.txt

# 追溯覆盖率检查（CI gate）
cargo test fr_ -- -Z unstable-options --format json | python3 scripts/verify-fr-coverage.py
```

## 7. 术语

| 术语 | 定义 |
|------|------|
| FR 验收测试 | 以 spec FR 为来源，断言用户可观测结果的测试 |
| 契约测试 | 断言接口一致性的测试 |
| 管道集成测试 | 断言端到端数据流连通的测试 |
| 可观测结果 | 用户通过 UI 或系统状态可直接感知的变化（如进程存在/不存在、注册表值、文件内容） |
| 追溯矩阵 | FR ID → 测试函数名 → 状态的映射表 |
