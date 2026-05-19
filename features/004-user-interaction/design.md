# Feature 004: 用户交互界面 — 设计

- **Feature**: 004-user-interaction
- **阶段**: `hf-design`
- **状态**: 草稿
- **日期**: 2026-05-12
- **上游输入**: `features/004-user-interaction/spec.md`
- **关联 ADR**: ADR-0002, ADR-0004, ADR-0006

## 1. 设计概述

Feature 004 是 GoGuo 的用户交互层，作为 Tauri 桌面应用的前端，承载 Feature 001~003 定义的全部交互语义。后端为 Rust（Tauri Commands），前端为 React + TypeScript（ADR-0006）。

## 2. 架构分层

```
┌─────────────────────────────────────┐
│         React Frontend              │
│  ┌─────────────────────────────┐    │
│  │       Page Components       │    │
│  │  Dashboard / Sites / Rules  │    │
│  │  Diag / Settings / Wizard   │    │
│  └──────────┬──────────────────┘    │
│  ┌──────────▼──────────────────┐    │
│  │      Shared Components      │    │
│  │  ConfirmDialog / NotifBar   │    │
│  │  SiteCard / RuleTable / ...  │    │
│  └──────────┬──────────────────┘    │
│  ┌──────────▼──────────────────┐    │
│  │    State Management (Zustand)│    │
│  │  ServiceStore / SiteStore   │    │
│  │  BaselineStore / NotifStore │    │
│  └──────────┬──────────────────┘    │
│  ┌──────────▼──────────────────┐    │
│  │     Tauri IPC Layer         │    │
│  │  invoke() → Rust Commands   │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
         ↕ Tauri IPC (invoke)
┌─────────────────────────────────────┐
│         Rust Backend                │
│  Tauri Commands → Core Services     │
│  (Feature 001/002/003 modules)      │
└─────────────────────────────────────┘
```

## 3. 状态管理

**选用 Zustand**：轻量、TypeScript 友好、无 Provider 包裹、冷启动友好。

### 3.1 Store 设计

| Store | 状态 | 来源 |
|-------|------|------|
| **ServiceStore** | 服务运行状态、baseline 状态、部署模式 | Feature 001 Commands |
| **BaselineStore** | 状态项列表、baseline 对比结果、恢复进度 | Feature 001 Commands |
| **SiteStore** | 目标站点列表、预设模板、站点定义 | Feature 003 Commands |
| **RuleStore** | 当前规则、规则预览、用户覆盖 | Feature 003 Commands |
| **DiagStore** | 站点可达性、诊断提示、节点池状态 | Feature 003 Commands |
| **NotifStore** | 应用内通知列表、系统通知能力 | 本地 + Feature 001/003 事件 |
| **WizardStore** | 首次引导流程状态（步骤、采集结果） | 本地 |

### 3.2 数据刷新策略

| 场景 | 刷新方式 | 频率 |
|------|----------|------|
| 服务状态 | Tauri Event 推送（Rust → Frontend） | 事件驱动 |
| 站点可达性 | Tauri Event 推送 | 探测完成时 |
| 恢复进度 | Tauri Event 推送 | 每项完成时 |
| 通知 | Tauri Event 推送 | 事件驱动 |
| 手动操作 | invoke() 调用后刷新 | 用户触发 |

Tauri Events（Rust 后端 → Frontend 推送）：

| Event | Payload | 触发时机 |
|-------|---------|----------|
| `service:status-changed` | `{ status, baseline_status }` | 服务状态变化 |
| `baseline:comparison-updated` | `{ items: [{id, match}] }` | baseline 对比完成 |
| `probe:site-result` | `{ site_id, reachable, time }` | 探测完成 |
| `rule:applied` | `{ success, rollback? }` | 规则应用完成 |
| `recovery:progress` | `{ completed, total, current_item }` | 恢复进度 |
| `recovery:completed` | `{ task_id, result }` | 恢复任务完成（成功/失败） |
| `recovery:failed` | `{ task_id, failed_items, explanation }` | 恢复任务失败 |
| `notification:new` | `{ type, message, timestamp }` | 通知产生 |
| `node-pool:changed` | `{ available, total, changed_node }` | 节点池变化 |

## 4. 页面结构设计

### 4.1 导航结构

```
┌──────────────────────────────────┐
│  GoGuo  [状态指示灯] [通知铃铛]    │
├────────┬─────────────────────────┤
│        │                         │
│ 仪表盘 │      主内容区域          │
│ 站点   │                         │
│ 规则   │                         │
│ 诊断   │                         │
│ 设置   │                         │
│        │                         │
├────────┴─────────────────────────┤
│  状态栏：服务状态 | baseline 状态  │
└──────────────────────────────────┘
```

**主路径验证（<= 2 步）**：

| 操作 | 步骤 | 验证 |
|------|------|------|
| 添加目标站点 | ① 点击"站点"→ ② 输入站点标识/选择模板 → 确认 | 2 步 + 确认 |
| 查看状态 | ① 打开应用 → 仪表盘直接展示 | 1 步 |
| 停止服务 | ① 点击状态栏"停止"→ ② 二次确认 | 2 步 |
| 查看诊断 | ① 点击"诊断"→ ② 查看不可达站点详情 | 2 步 |

### 4.2 仪表盘页面

展示内容：
- 服务运行状态（运行中/已停止/异常）+ baseline 状态（已偏离时提供"立即恢复"行动入口）
- 部署模式（对应 Feature 002 DeploymentMode：仅 Windows / 仅 WSL / 仅 Linux / Windows + WSL）
- 目标站点可达性摘要（语义化：全部可用 / N 个需要关注）
- 最近通知列表（最新 5 条）
- 快捷操作：启动/停止服务

### 4.3 站点管理页面

- 站点列表（当前已添加的目标站点）
- 添加站点：输入站点标识或域名 → 展开域名预览 → 确认
- 预设模板：开发者套装 / 办公套装，一键应用
- 删除站点：确认后移除

### 4.4 规则预览页面

- 当前生效的规则列表（按站点分组）
- 待应用的规则变更预览
- 批量预览（按站点分组，可展开/折叠）（FR-2.5.1-R4）
- 用户自定义规则覆盖标记

### 4.5 诊断页面

- 每个目标站点的可达性状态 + 响应时间
- 不可达站点：五要素诊断提示 + 建议动作
- 节点池状态：可用数/总数 + 各节点元数据
- 恢复进度实时展示

### 4.6 设置页面

- 部署模式选择（协同/仅 Windows/仅 WSL/仅 Linux）
- 探测间隔配置
- 通知偏好
- 关于信息

### 4.7 首次引导流程（Wizard）

```
Step 1: 欢迎 + 安装后网络评估
         ↓
Step 2: 展示当前网络状态（分类展示）
         ↓
Step 3: 手工调整引导（step by step）
        - 逐项展示不理想的状态项
        - 可执行的命令行命令（一键复制）
        - 系统页面操作指引
        - 可选：触发重新采集
         ↓
Step 4: 确认 baseline（二次确认）
         ↓
Step 5: 选择部署模式
         ↓
Step 6: 选择目标站点（预设模板 / 手动添加）
         ↓
Step 7: 完成
```

## 5. 组件设计

### 5.1 核心交互组件

| 组件 | 功能 | 复用位置 |
|------|------|----------|
| `ConfirmDialog` | 二次确认对话框（baseline 确认、规则应用、服务启停） | 全局 |
| `StatusBadge` | 状态指示（运行中/已停止/异常/可达/不可达/探测中） | 仪表盘、诊断 |
| `SiteCard` | 站点卡片（名称、状态、可达性、操作） | 站点管理、诊断 |
| `RuleTable` | 规则列表（域名、策略、来源、覆盖标记） | 规则预览 |
| `DiagPanel` | 诊断面板（五要素提示、建议动作、可执行命令） | 诊断 |
| `NodePoolTable` | 节点池表格（名称、状态、入池时间、检测时间） | 诊断 |
| `NotifBar` | 通知栏（时间戳、类型、内容、操作） | 全局 |
| `Wizard` | 首次引导向导（多步骤、进度条、step by step 指引） | 首次启动 |
| `CodeBlock` | 可执行命令展示（语法高亮 + 一键复制） | 手工调整引导 |
| `RecoveryOverlay` | 恢复蒙层（全屏半透明 + 居中进度卡片，屏蔽所有导航和操作） | 恢复/续跑期间 |
| `RecoveryAckDialog` | 恢复失败确认对话框（展示失败项 + "确认已修复"/"重新恢复"按钮） | 恢复失败后 |
| `AuditLogTable` | 审计日志表格（分页加载、日期/类型过滤） | 诊断-审计 |

### 5.2 确认机制

敏感操作统一使用 `ConfirmDialog`：

| 操作 | 确认文案 | 来源 |
|------|----------|------|
| Baseline 确认 | "确认当前网络状态为可用基线？" | Feature 001 FR-2.2.2-R3 |
| 服务停止 | "停止服务将恢复到 baseline，确认？" | Feature 001 FR-2.4.1 |
| 规则应用 | "将应用以下代理规则，确认？" | Feature 003 FR-2.6.1-R1 |
| 站点删除 | "移除该目标站点将重新生成规则，确认？" | Feature 003 |
| 部署模式切换 | "切换部署模式将改变监控范围，确认？" | Feature 002 |

## 6. 通知系统

### 6.1 应用内通知

```typescript
interface AppNotification {
  id: string;
  type: 'rule-rollback' | 'recovery' | 'audit-change' | 'node-pool';
  title: string;
  message: string;
  timestamp: Date;
  actions?: { label: string; command: string }[];
}
```

**通知触发事件**：

| 事件 | 通知类型 | 优先级 |
|------|----------|--------|
| 规则回退 | rule-rollback | 高 |
| 恢复动作（成功/失败） | recovery | 中 |
| 审计变更 | audit-change | 低 |
| 节点移除/恢复 | node-pool | 中 |

### 6.2 系统通知

- 重要通知（规则回退、恢复失败）同时通过系统通知推送
- 使用 Tauri notification API
- 系统通知不可用时，应用内通知作为降级方案
- WSL/Linux 侧通知降级处理（OP-3）

## 7. 离线行为（OP-1）

- 后端未运行时，UI 必须能启动并展示离线状态（NFR-3.2-1）
- 离线状态下：
  - 展示"服务未运行"状态
  - 展示上次已知的 baseline 和站点状态（从本地存储读取）
  - 提供"启动服务"按钮
- 后端上线后自动恢复连接并刷新状态（NFR-3.2-2）

## 8. 性能设计

### 8.1 冷启动优化（3s 目标）

| 阶段 | 策略 |
|------|------|
| Tauri 启动 | 最小化 Rust 初始化（延迟加载非关键模块） |
| 前端加载 | Vite code splitting + 首页懒加载 |
| 首屏渲染 | 仪表盘优先渲染，使用缓存的上次状态 |
| API 调用 | 首屏使用本地缓存，后台异步刷新 |

### 8.2 UI 刷新延迟（2s 目标）

- Tauri Event 推送替代轮询
- Zustand store 细粒度更新（不重渲染整页）
- 列表数据虚拟化（站点/规则数量增长时）

## 9. 约束与不变量

- **C1**: UI 全部数据来源于本地 API（Tauri Commands），不发起远程请求（CON-3）
- **C2**: Windows、WSL 和 Linux 各侧 UI 一致（CON-2）
- **C3**: 主路径操作不超过 2 步（CON-4）
- **C4**: 当前阶段无用户登录/鉴权，启动用户即管理员（FR-2.1.1-R6）
- **C5**: 后端未运行时 UI 不崩溃（NFR-3.2-1）

## 10. 风险与缓解

| 风险 | 缓解 |
|------|------|
| WSL/Linux 侧 WebKitGTK 不可用 | 检测并提示安装 + 提供命令行降级 |
| 系统通知在 WSL 不可用 | 降级为仅应用内通知 |
| 冷启动超过 3s | 首屏最小渲染 + 延迟加载 + 缓存 |
| 跨平台 UI 样式差异 | 统一 CSS 变量 + shadcn/ui 跨平台测试 |

## 11. 测试策略

| 测试 | 方式 |
|------|------|
| 组件单元测试 | React Testing Library |
| 主路径步骤计数 | 验证每个核心操作 ≤ 2 步 |
| 跨平台一致性 | Windows + WSL 截图对比 |
| 冷启动计时 | 自动化启动计时（≤ 3s） |
| 离线行为 | 后端未启动时验证 UI 行为 |
| 通知推送 | 触发各事件类型，验证应用内 + 系统通知 |
| 二次确认 | 验证每个敏感操作需确认 |
| 状态一致性 | UI 显示与 Tauri Command 返回数据对比 |

## 12. 开放问题处理

| OP ID | 设计阶段处理 |
|-------|-------------|
| OP-1 | 离线状态展示上次已知数据 + "服务未运行"状态 + 启动按钮 |
| OP-2 | 桌面框架已确定为 Tauri（ADR-0002） |
| OP-3 | WSL/Linux 系统通知不可用时降级为仅应用内通知 |

## 13. 跨 Feature 修订记录

### 13.1 基于 Feature 001 design.md 标注修订（2026-05-14）

| 标注 ID | 变更 | 本 Feature 影响 |
|---------|------|----------------|
| id:01 | WSL/Linux 拆为 WslAdapter + LinuxAdapter | 部署模式选择从 3 项增至 4 项（+仅 Linux） |
| id:05 | 审计记录查询需分页 | 新增 `AuditLogTable` 组件，`get_audit_log` 命令增加 `offset/limit/filter` 参数 |
| id:06 | 续跑期间 UI 蒙层限制 | 新增 `RecoveryOverlay` 组件（全屏蒙层 + 居中进度卡片，屏蔽导航和操作） |
| id:07 | 恢复任务状态机 | 新增 `RecoveryAckDialog` 组件（失败后"确认已修复"/"重新恢复"），新增 `recovery:completed`/`recovery:failed` 事件 |
