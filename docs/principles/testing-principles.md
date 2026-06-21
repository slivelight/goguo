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

## 8. L1~L5 自动化测试设计强制规范

> 来源：F115（2026-06-18 立项）。AGENTS.md §7 入口指向本节。
> 配套：`docs/test-level-matrix.md`（等级分工）、`docs/principles/test-design-section-template.md`（§N 章节模板）。

### 8.1 强制条款（与 AGENTS.md §7.1 摘要一致）

- **条款 1**：`hf-design` 必须输出 `§N L1~L5 自动化测试设计` 章节（模板：`docs/principles/test-design-section-template.md`），作为 design.md 必填章节。
- **条款 2**：该章节未通过 review **不得进入** `hf-tasks` 阶段——即编码启动前必须完成测试设计。
- **条款 3**：章节必须同时覆盖 **UX 能力（L4/L5）** 与 **非 UI 能力（L1/L2/L3）**，对每条 FR 给出测试用例 + 数据 + 脚本入口。
- **条款 4**：`hf-finalize` 必须验证该章节中所有声明的测试已实现且通过，否则不通过完成门。

### 8.2 HF 全流程检查点

| HF 阶段 | 检查点 | 责任人 |
|---------|-------|-------|
| `hf-specify` | 矩阵为本 Feature 占行（FR ID + 等级标注，函数名允许 `<TBD by design>`） | spec 作者 |
| `hf-design` | design.md 含完整 §N "L1~L5 自动化测试设计"章节；矩阵函数名填齐；测试用例 + 数据 + 脚本设计完成 | design 作者 |
| `hf-tasks` | tasks.md 拆解时含每条测试的实施 task（按 §N.5 顺序） | tasks 作者 |
| `hf-test-driven-dev` | 按 §N.5 RED-GREEN 执行；不允许跳过 L1~L5 任意层 | 实施者 |
| `hf-finalize` | 验证 §N 中所有声明的测试已实现且通过；矩阵本 Feature 行内 `<TBD` 计数 = 0 | finalize 审查 |

### 8.3 显式豁免清单（不要求补 §N 章节）

- **F109 / F110**：F115 立项前已进入 design 阶段，回溯补章节成本高于收益；按"原 spec/design 为 authority source"模式推进，待启动后按"新启动 fix Feature"对待时若 review 认为必要再补
- **F114**：UI E2E PoC，产物为 PoC 报告，不进入正式 Feature 工件流
- **F115**：本规范的建立 Feature，避免循环依赖
- **F101~F106**：v0.1.0 审计发现的待办修复项，尚未启动；启动时按"新启动 fix Feature"对待（即需补 §N 章节）

> 豁免清单仅豁免"补 §N 章节"硬要求；其它测试纪律（三层测试方法论、RED-GREEN）仍适用。

### 8.4 L1~L5 等级决策原则

8 条能力特征 → 测试等级的映射原则详见本文件 [§9](#9-l1l5-等级决策原则)（自包含参考）+ [`docs/test-level-matrix.md`](../test-level-matrix.md) §1（权威矩阵视图）。本节为强制规范与原则之间的桥梁，不重复条款。

### 8.5 矩阵执行约束

- **L4 能力不重复在 e2e/ 实现**（避免冗余；L4 = vitest+RTL 已覆盖组件级行为）
- **L5 能力必须有 e2e spec 承接**（端到端真实环境不可被低层替代）
- 矩阵 TBD 阈值：本 Feature finalize 时，本 Feature 行内 `<TBD` 计数 = 0（spec FR-2.3.1-R3a 阶段 2）

---

## 9. L1~L5 等级决策原则

> **来源**：F115 spec FR-2.3.1-R4 / design §4.2。
> **权威矩阵视图**：[`docs/test-level-matrix.md`](../test-level-matrix.md) §1（本节为 testing-principles 自包含重述，二者保持一致；如有差异以 test-level-matrix §1 为准）。
> **互引**：本节被 [§8](#8-l1l5-自动化测试设计强制规范) 强制规范引用；本节引用 §8 强制规范作为应用约束。

设计 `§N L1~L5 自动化测试设计` 章节（[test-design-section-template.md](./test-design-section-template.md)）时，每条 FR 的等级标注必须依据下表 8 条原则：

| 能力特征 | 等级 | 依据 |
|---------|------|------|
| 跨进程数据流（IPC → 后端 → 响应 → UI 更新） | **L5** | 端到端真实环境，任何低层都无法替代 |
| Tauri 事件订阅与前端响应 | **L5** | webview 特性，需真实运行时验证 |
| Tauri webview 特性（X11/Wayland、IPC 时序、WebKitGTK） | **L5** | 平台特性，单元测试无法模拟 |
| 跨页面状态同步（Zustand store 之外） | **L5** | 集成行为，组件级测试不可见 |
| 单组件渲染、props、内部状态机 | **L4** | 组件级隔离，vitest+RTL 足够 |
| 单 Rust 模块纯函数、数据结构 | **L1** | 单元，无 IO 无副作用 |
| Rust trait 一致性、DTO 往返 | **L3** | 契约层，跨模块但非端到端 |
| FR 级可观测行为（不依赖 UI） | **L2** | FR 验收，断言用户可感知结果 |

### 9.1 应用约束（与 §8.5 一致）

- 同一能力在多等级出现时，**取最高等级**作为必须覆盖项（例如：跨 IPC 数据流既可写 L2 断言也可写 L5 e2e，必须至少有 L5）
- **L4 能力不重复在 e2e/ 实现**（避免冗余）
- **L5 能力必须有 e2e spec 承接**
- 等级原则可演进（spec FR-2.3.1-R4 允许 design 阶段调整，但需在 design.md 说明偏离理由）
