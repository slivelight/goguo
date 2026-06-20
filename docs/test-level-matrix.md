# 测试等级矩阵（L1~L5）

> **维护时机**：每个 Feature 在 `hf-specify` 阶段占行；`hf-design` 阶段填测试函数名；`hf-finalize` 阶段验证 TBD 计数。
>
> **配套文档**：FR 测试函数 1:1 追溯见 [test-trace-matrix.md](./test-trace-matrix.md)；等级强制规范见 [principles/testing-principles.md](./principles/testing-principles.md) §"L1~L5 自动化测试设计强制规范"。
>
> **来源**：F115 FR-2.3.1 / design §4。

---

## 1. 等级决策原则（spec FR-2.3.1-R4）

| 能力特征 | 等级 | 依据 |
|---------|------|------|
| 跨进程数据流（IPC → 后端 → 响应 → UI 更新） | **L5** | 端到端真实环境 |
| Tauri 事件订阅与前端响应 | **L5** | webview 特性 |
| Tauri webview 特性（X11/Wayland、IPC 时序、WebKitGTK） | **L5** | 平台特性 |
| 跨页面状态同步（Zustand store 之外） | **L5** | 集成行为 |
| 单组件渲染、props、内部状态机 | **L4** | 组件级隔离 |
| 单 Rust 模块纯函数、数据结构 | **L1** | 单元 |
| Rust trait 一致性、DTO 往返 | **L3** | 契约 |
| FR 级可观测行为（不依赖 UI） | **L2** | 验收 |

---

## 2. 矩阵

### 2.1 F201: 首次安装引导与基线确认闭环

> **填充策略**（design §4.3）：F115 阶段为每条 FR 占行 + 等级标注；L2 列对**继承自 F001~F004 的测试**填具体函数名（来自 trace-matrix），对**全新增的 FR** 标 `<TBD by F201 design>`；其它列若 F201 design 未启动，等级标注到位即可。

| 能力 ID | 能力描述 | 关联 FR | L1（Rust 单测） | L2（FR 验收） | L3（契约/管道） | L4（vitest+RTL） | L5（e2e） | 依据 |
|---------|---------|--------|---------------|--------------|----------------|----------------|----------|------|
| F201-FR-1.0 | 启动自动采集 baseline + 持久化 | FR-1.0 | `BaselineStorage::save_auto_baseline` 单测 | `fr_2_1_1_snapshot_before_modification`（继承 F001） | N/A | N/A | N/A | 后台一次性动作，无 UI；L2 已覆盖可观测结果 |
| F201-FR-1.1 | Wizard Step 2 评估结果展示 | FR-1.1 | N/A（评估分类逻辑已在 F001 单测） | `fr_2_1_2_assessment_readonly`（继承 F001） | N/A | `<TBD by F201 design>`：Step2 分类展示渲染 | `<TBD by F201 design>`：e2e:wizard-eval-display | UI 行为必须 L4+L5 |
| F201-FR-1.2 | Wizard Step 3 手工调整引导 | FR-1.2 | `<TBD by F201 design>`：`get_suboptimal_items` 单测 | `<TBD by F201 design>`：FR-1.2 验收 | N/A | `<TBD by F201 design>`：AdjustmentItem 列表渲染 | `<TBD by F201 design>`：e2e:wizard-adjustment | 全新增（F110 G110-3） |
| F201-FR-1.3 | Wizard Step 4 baseline 确认 + 调整实施 | FR-1.3 | `<TBD by F201 design>`：`apply_adjustments_batch` 单测 | `fr_2_2_2_confirmation_interaction`（继承 F001） | `<TBD by F201 design>`：管道 测 apply→persist→audit | `<TBD by F201 design>`：ConfirmDialog 渲染 | `<TBD by F201 design>`：e2e:wizard-confirm | 跨 IPC 数据流必须 L5 |
| F201-FR-1.4 | Wizard Step 5 部署模式选择 | FR-1.4 | `DeploymentManager::detect` 单测（F001 已有） | `fr_2_9_1_deployment_identification`（继承 F001） | N/A | `<TBD by F201 design>`：4 模式卡片渲染 | `<TBD by F201 design>`：e2e:wizard-deployment | UI + 持久化 |
| F201-FR-1.5 | Wizard Step 2 / Dashboard 可达性展示 | FR-1.5 | `<TBD by F110 G110-1>`：`RealProbeClient` 单测 | `<TBD by F201 design>`：FR-1.5 验收 | N/A | `<TBD by F201 design>`：可达性摘要卡片渲染 | `<TBD by F201 design>`：e2e:reachability-display | UI + 真实探测 |
| F201-FR-1.6 | Wizard Step 6 站点选择（模板） | FR-1.6 | N/A（apply_preset_template 已在 F003 单测） | `fr_2_4_1_site_add_remove`（继承 F004） | N/A | `<TBD by F201 design>`：模板选择 + 勾选渲染 | `<TBD by F201 design>`：e2e:wizard-site-selection | UI 行为 |
| F201-FR-1.7 | baseline 重置（设置页） | FR-1.7 | `<TBD by F201 design>`：`reset_baseline` 单测 | `<TBD by F201 design>`：FR-1.7 验收（含前置守卫） | N/A | `<TBD by F201 design>`：前置守卫提示渲染 | N/A（设置页非 webview 关键路径，L4 足够） | 全新增 |
| F201-FR-1.8 | baseline 清除（设置页） | FR-1.8 | `<TBD by F201 design>`：`clear_baseline` 单测 | `<TBD by F201 design>`：FR-1.8 验收 | `<TBD by F201 design>`：管道 clear→stop_mihomo→restore_proxy | `<TBD by F201 design>`：二次确认 + 状态更新渲染 | `<TBD by F201 design>`：e2e:clear-baseline | 全新增 + 跨进程管道 |

#### 2.1.1 阶段 1 完整性自检（spec FR-2.3.1-R3a，design §4.3.2/4.3.3）

| 指标 | 要求来源 | 要求 | 实际 | 结果 |
|------|---------|------|------|------|
| 行数 | spec FR-2.3.1-R3 | ≥ F201 spec FR 总数 × 0.6 = 9 × 0.6 = 5.4 | 9 行 | ✅ |
| 等级标注完整 | spec FR-2.3.1-R3a 阶段 1 | 每行 L1~L5 列至少有等级标注（含 N/A） | 9 行全标注 | ✅ |
| **L2 列** `<TBD` 计数 | design §4.3.3 自定（**非 spec 强制**）| ≤ 9 × 0.4 = 3.6 | **4 个**（FR-1.2 / FR-1.5 / FR-1.7 / FR-1.8）| ⚠️ 4 > 3.6，见下分析 |
| **全表** `<TBD` 计数 | spec FR-2.3.1-R3a 阶段 2 | = 0（**F201 finalize 考核**，非 F115）| ≈ 26 | ⏳ 待 F201 design 启动后填入 |

> **spec 阶段 1 通过条件**（FR-2.3.1-R3a 原文）：F201 所有 FR 占行 + 每行 L1~L5 责任列至少有等级标注 + 函数名列允许 TBD。**本矩阵已满足**。
>
> **design §4.3.3 自定 L2 阈值（≤3.6）分析**：
> - design 原汇总写"3 个 TBD（FR-1.2/1.7/1.8），达标"——**算术错误**：逐行分布显示 FR-1.5 L2 也是 TBD（探测非目标站点可达性，F201 全新增能力，无 F001~F004 继承测试可填）。
> - 修正后 L2 列 TBD = 4，超 design 自定阈值 3.6（4/9 = 44% vs 40%）。
> - **不影响 spec 阶段 1 通过**（spec 无 L2 阈值）；design §4.3.3 此条作为质量参考而非硬验收，待 F201 design 启动后这 4 个 L2 TBD 中至少 FR-1.5/1.7/1.8 可由 F201 新增测试填入。
> - **回写建议**：F115 finalize 时同步回写 design §4.3.3 修正算术错误（3→4 + 补 FR-1.5）。

逐行 TBD 分布：

| 能力 ID | L1 | L2 | L3 | L4 | L5 | 合计 |
|---------|----|----|----|----|----|------|
| F201-FR-1.0 | 0 | 0 | 0 | 0 | 0 | 0 |
| F201-FR-1.1 | 0 | 0 | 0 | 1 | 1 | 2 |
| F201-FR-1.2 | 1 | **1** | 0 | 1 | 1 | 4 |
| F201-FR-1.3 | 1 | 0 | 1 | 1 | 1 | 4 |
| F201-FR-1.4 | 0 | 0 | 0 | 1 | 1 | 2 |
| F201-FR-1.5 | 1 | **1** | 0 | 1 | 1 | 4 |
| F201-FR-1.6 | 0 | 0 | 0 | 1 | 1 | 2 |
| F201-FR-1.7 | 1 | **1** | 0 | 1 | 0 | 3 |
| F201-FR-1.8 | 1 | **1** | 1 | 1 | 1 | 5 |
| **合计** | 5 | **4** | 2 | 7 | 7 | **≈26** |

### 2.2 F202~F205

<TBD by 各 Feature design 阶段>

---

## 3. 矩阵执行约束（spec FR-2.3.2）

- **L4 能力不重复在 e2e/ 实现**（避免冗余，design §4.4）
- **L5 能力必须有 e2e spec 承接**（spec FR-2.3.2-R1）
- 详细条款落盘于 [principles/testing-principles.md](./principles/testing-principles.md) §"L1~L5 自动化测试设计强制规范"

---

## 4. 矩阵更新规则

| HF 阶段 | 动作 | 责任角色 |
|---------|------|---------|
| `hf-specify` | 占行 + 等级标注（L1~L5 列每列至少有标注或 N/A） | spec 作者 |
| `hf-design` | 填测试函数名（从 trace-matrix 对齐继承测试 + 新增测试命名） | design 作者 |
| `hf-finalize` | grep `<TBD` 计数 = 0（本 Feature 行内）；同步更新 trace-matrix | finalize 守门 |

**关联 lint**：`e2e/scripts/lint-specs.mjs`（FR-2.4.2-R1）校验 e2e spec 路径合规，不直接校验本矩阵——矩阵 TBD 计数由 finalize 阶段 grep 核对。
