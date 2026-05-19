# Feature 004: 用户交互界面 — 任务计划

- 状态: 草稿
- 主题: user-interaction
- 阶段: `hf-tasks`
- 上游输入: `features/004-user-interaction/spec.md`、`features/004-user-interaction/design.md`、`features/004-user-interaction/ui-design.md`
- **前置依赖**: Feature 001~003 Tauri Commands 完成（后端 IPC 接口就绪）

## 1. 概述

本任务计划将 Feature 004 设计转化为可执行任务。Feature 004 是 GoGuo 的用户交互层，作为 Tauri 桌面应用前端，承载 F001~F003 全部交互语义。

**实现范围**：React + TypeScript 前端（7 个 Zustand Store、7 个页面、13 个核心组件、首次引导 Wizard）、Tauri IPC 层、通知系统、离线行为、冷启动优化。

## 2. 里程碑

| 里程碑 | 目标 | 退出标准 | 对应设计章节 |
|--------|------|----------|-------------|
| **M1: 前端基础设施** | React + TS 项目结构、路由、Tauri IPC 封装 | 空页面可渲染，invoke 可调用 | design §2 |
| **M2: Zustand Store 层** | 7 个 Store + Tauri Event 监听 | 单元测试通过：状态更新+事件响应 | design §3 |
| **M3: 共享组件** | ConfirmDialog、StatusBadge、NotifBar 等 13 个 | 组件测试通过 | design §5.1 |
| **M4: 仪表盘** | Dashboard 页面 + 状态卡片 | 组件测试+手动验证 | design §4.2 |
| **M5: 站点管理** | Sites 页面 + 添加/删除/模板 | 交互测试通过 | design §4.3 |
| **M6: 规则预览** | Rules 页面 + 批量预览 | 交互测试通过 | design §4.4 |
| **M7: 诊断页** | Diagnostics + 节点池表格 | 交互测试通过 | design §4.5 |
| **M8: 设置页** | Settings + 部署模式 4 档 | 交互测试通过 | design §4.6 |
| **M9: 首次引导** | Wizard 7 步流程 | 交互测试通过 | design §4.7 |
| **M10: 通知+离线+性能** | 通知系统、离线行为、冷启动 ≤3s | 集成验证通过 | design §6,7,8 |

## 3. 文件 / 工件影响图

```
src/                              # React 前端
  app/
    layout.tsx                    # 全局布局（侧边栏+状态栏）
    page.tsx                      # 默认重定向到仪表盘
    dashboard/page.tsx
    sites/page.tsx
    rules/page.tsx
    diagnostics/page.tsx
    settings/page.tsx
    wizard/page.tsx
  components/
    layout/
      Sidebar.tsx
      StatusBar.tsx
      Header.tsx
    shared/
      ConfirmDialog.tsx
      StatusBadge.tsx
      NotifBar.tsx
      CodeBlock.tsx
      RecoveryOverlay.tsx
      RecoveryAckDialog.tsx
    dashboard/
      ServiceCard.tsx
      BaselineCard.tsx
      DeploymentModeCard.tsx
      ReachabilitySummary.tsx
    sites/
      SiteCard.tsx
      AddSiteDialog.tsx
      PresetTemplateSelector.tsx
      InlineDiagPanel.tsx
    rules/
      RuleTable.tsx
      RulePreview.tsx
    diagnostics/
      DiagPanel.tsx
      NodePoolTable.tsx
      AuditLogTable.tsx
    wizard/
      Wizard.tsx
      WizardStep.tsx
  stores/
    service-store.ts
    baseline-store.ts
    site-store.ts
    rule-store.ts
    diag-store.ts
    notif-store.ts
    wizard-store.ts
  lib/
    tauri-ipc.ts                  # Tauri invoke 封装
    events.ts                     # Tauri Event 监听封装
    types.ts                      # 共享类型
  hooks/
    use-tauri-event.ts
    use-offline.ts
  styles/
    globals.css                   # 全局样式 + CSS 变量
src-tauri/
  capabilities/                   # Tauri 权限配置
```

## 4. 需求与设计追溯

| 需求 ID | 设计章节 | 任务覆盖 |
|---------|---------|---------|
| FR-2.1.1-R6 (无鉴权管理员) | §9 C4 | T1.2 |
| FR-2.2.1-R3 (手工调整引导) | §4.7 Wizard Step 3 | T9.1 |
| FR-2.2.2-R3 (确认 baseline) | §5.2 ConfirmDialog | T3.1 |
| FR-2.4.1 (服务停止确认) | §5.2 | T3.1 |
| FR-2.5.1-R4 (批量预览) | §4.4 | T6.1 |
| FR-2.6.1-R1 (规则应用确认) | §5.2 | T3.1 |
| NFR-3.1-1 (冷启动 ≤3s) | §8.1 | T10.2 |
| NFR-3.2-1/2 (离线行为) | §7 | T10.1 |
| CON-2 (跨平台一致) | §9 C2 | T10.3 |
| CON-4 (主路径 ≤2 步) | §4.1 | M4~M8 验证 |

## 5. 任务拆解

### T1.1: 前端项目结构与路由

- **目标**: 建立 React + TypeScript + pnpm 项目结构，配置路由（仪表盘/站点/规则/诊断/设置/引导），全局布局（侧边栏+状态栏+Header）
- **Acceptance**: `pnpm dev` 启动，6 个路由页面可导航，侧边栏可切换
- **依赖**: Feature 001 T1.1（Tauri 项目脚手架）
- **Ready When**: F001 脚手架完成
- **初始队列状态**: pending
- **Selection Priority**: 1
- **Files / 触碰工件**: `src/app/layout.tsx`, `src/app/page.tsx`, `src/app/*/page.tsx`, `src/components/layout/Sidebar.tsx`, `src/components/layout/StatusBar.tsx`, `src/components/layout/Header.tsx`, `src/styles/globals.css`
- **测试设计种子**: 路由导航测试：每个路径渲染对应页面组件；侧边栏高亮当前页；状态栏显示占位内容
- **Verify**: `pnpm build` 成功，手动导航验证
- **预期证据**: 构建成功，6 页可导航
- **完成条件**: 前端结构完整，路由正确，布局组件渲染

### T1.2: Tauri IPC 封装 + 类型定义

- **目标**: 封装 Tauri invoke() 调用（F001~F003 全部 Commands），定义前端共享类型，封装 Tauri Event 监听
- **Acceptance**: 所有 Command 有对应 TypeScript 函数；所有 Event 有类型安全的监听 hook
- **依赖**: Feature 001~003 T9.1/T6.1/T8.1（Tauri Commands 定义完成）
- **Ready When**: 后端 Commands 接口定义完成（可并行开发，使用 mock）
- **初始队列状态**: pending
- **Selection Priority**: 1
- **Files / 触碰工件**: `src/lib/tauri-ipc.ts`, `src/lib/events.ts`, `src/lib/types.ts`, `src/hooks/use-tauri-event.ts`
- **测试设计种子**: Mock invoke→验证参数传递和返回值类型；Event 监听→验证 payload 解析
- **Verify**: `pnpm test -- --testPathPattern=lib/tauri-ipc`
- **预期证据**: IPC 封装测试通过
- **完成条件**: 全部 Commands 和 Events 有类型安全的封装

### T2.1a: 核心 Zustand Store（3 个）

- **目标**: 实现 ServiceStore、BaselineStore、NotifStore——与 F001 交互最紧密的 3 个核心 Store
- **Acceptance**: 每个 Store 状态初始化正确；action 调用更新状态；Tauri Event（service:status-changed / baseline:comparison-updated / notification:new）触发状态更新
- **依赖**: T1.2
- **Ready When**: IPC 封装完成
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src/stores/service-store.ts`, `src/stores/baseline-store.ts`, `src/stores/notif-store.ts`
- **测试设计种子**: ServiceStore：初始→service:status-changed event→状态更新断言；BaselineStore：初始→baseline:comparison-updated→偏离项标记；NotifStore：空列表→notification:new→追加+类型断言
- **Verify**: `pnpm test -- --testPathPattern=stores/(service|baseline|notif)`
- **预期证据**: 3 个核心 Store 测试通过
- **完成条件**: 3 个核心 Store 实现正确，Event 响应正确

### T2.1b: 扩展 Zustand Store（4 个）

- **目标**: 实现 SiteStore、RuleStore、DiagStore、WizardStore——与 F002/F003 交互的扩展 Store
- **Acceptance**: 每个 Store 状态初始化正确；action 调用更新状态；Tauri Event（probe:site-result / rule:applied / node-pool:changed）触发状态更新
- **依赖**: T1.2, T2.1a
- **Ready When**: 核心 Store 模式确立
- **初始队列状态**: pending
- **Selection Priority**: 2
- **Files / 触碰工件**: `src/stores/site-store.ts`, `src/stores/rule-store.ts`, `src/stores/diag-store.ts`, `src/stores/wizard-store.ts`
- **测试设计种子**: SiteStore：初始→add/remove→列表更新；RuleStore：初始→preview_rules→预览数据填充；DiagStore：初始→probe:site-result→可达性更新；WizardStore：步骤导航测试
- **Verify**: `pnpm test -- --testPathPattern=stores/(site|rule|diag|wizard)`
- **预期证据**: 4 个扩展 Store 测试通过
- **完成条件**: 4 个扩展 Store 实现正确，Event 响应正确

### T3.1: 核心共享组件

- **目标**: 实现 ConfirmDialog（含后果可视化）、StatusBadge、NotifBar（含历史归档入口）、CodeBlock、RecoveryOverlay、RecoveryAckDialog
- **Acceptance**: ConfirmDialog 展示"当前值→恢复后"；RecoveryOverlay 全屏蒙层；NotifBar 支持历史查看
- **依赖**: T2.1
- **Ready When**: Store 层完成
- **初始队列状态**: pending
- **Selection Priority**: 3
- **Files / 触碰工件**: `src/components/shared/*.tsx`（6 个文件）
- **测试设计种子**: ConfirmDialog 打开/关闭/确认回调；StatusBadge 各状态渲染；RecoveryOverlay 遮罩层+进度卡片；RecoveryAckDialog 失败项展示
- **Verify**: `pnpm test -- --testPathPattern=components/shared/`
- **预期证据**: 共享组件测试通过
- **完成条件**: 6 个共享组件实现正确

### T4.1: 仪表盘页面

- **目标**: 实现 Dashboard 页面：服务状态卡片、Baseline 状态卡片、部署模式卡片、语义化可达性摘要、最近通知、快捷操作
- **Acceptance**: 3 个状态卡片正确渲染；可达性语义化展示（"全部可用"/"N 个需要关注"）；"立即恢复"行动入口
- **依赖**: T3.1
- **Ready When**: 共享组件完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src/app/dashboard/page.tsx`, `src/components/dashboard/*.tsx`
- **测试设计种子**: 各卡片状态组合渲染（运行中/已停止/异常）；可达性摘要数字正确；快捷操作按钮触发 Store action
- **Verify**: `pnpm test -- --testPathPattern=dashboard/`
- **预期证据**: 仪表盘测试通过
- **完成条件**: 仪表盘完整实现，状态展示正确

### T5.1: 站点管理页面

- **目标**: 实现 Sites 页面：站点列表、添加站点（意图导向输入+域名预览）、预设模板（下拉预览）、删除确认、内嵌诊断面板
- **Acceptance**: 站点列表正确展示；添加流程：输入→预览→确认；模板下拉预览可用；不可达站点展示内嵌诊断
- **依赖**: T3.1
- **Ready When**: 共享组件完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src/app/sites/page.tsx`, `src/components/sites/*.tsx`
- **测试设计种子**: 空列表→添加站点→验证列表更新；预设模板选择→验证站点添加；不可达站点→内嵌诊断面板展示；删除→确认对话框
- **Verify**: `pnpm test -- --testPathPattern=sites/`
- **预期证据**: 站点管理测试通过
- **完成条件**: 站点 CRUD + 模板 + 诊断面板完整

### T6.1: 规则预览页面

- **目标**: 实现 Rules 页面：按站点分组规则列表、批量预览（可展开/折叠）、用户自定义覆盖标记、语义化统计
- **Acceptance**: 规则按站点分组展示；批量预览可展开/折叠；统计语义化（"为 N 个网站配置了访问方式"）
- **依赖**: T3.1
- **Ready When**: 共享组件完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src/app/rules/page.tsx`, `src/components/rules/*.tsx`
- **测试设计种子**: 多站点规则分组展示；折叠/展开交互；用户覆盖标记；统计数字与站点数一致
- **Verify**: `pnpm test -- --testPathPattern=rules/`
- **预期证据**: 规则预览测试通过
- **完成条件**: 规则预览+批量预览+覆盖标记正确

### T7.1: 诊断页面

- **目标**: 实现 Diagnostics 页面：站点可达性状态+响应时间、不可达站点五要素诊断、节点池表格（元数据）、审计日志表格（分页+过滤）、恢复进度实时展示
- **Acceptance**: 诊断面板展示五要素；节点池表格展示元数据；审计日志分页加载正确
- **依赖**: T3.1
- **Ready When**: 共享组件完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src/app/diagnostics/page.tsx`, `src/components/diagnostics/*.tsx`
- **测试设计种子**: 可达/不可达站点渲染差异；五要素诊断提示内容；节点池表格分页；审计日志日期/类型过滤
- **Verify**: `pnpm test -- --testPathPattern=diagnostics/`
- **预期证据**: 诊断页测试通过
- **完成条件**: 诊断+节点池+审计日志完整

### T8.1: 设置页面

- **目标**: 实现 Settings 页面：部署模式选择（4 档，需确认切换）、探测间隔配置、通知偏好、关于信息
- **Acceptance**: 4 种部署模式可切换（需确认）；配置修改持久化
- **依赖**: T3.1
- **Ready When**: 共享组件完成
- **初始队列状态**: pending
- **Selection Priority**: 4
- **Files / 触碰工件**: `src/app/settings/page.tsx`
- **测试设计种子**: 4 种部署模式渲染；切换→确认对话框→配置保存；探测间隔输入验证
- **Verify**: `pnpm test -- --testPathPattern=settings/`
- **预期证据**: 设置页测试通过
- **完成条件**: 设置页功能完整

### T9.1: 首次引导 Wizard

- **目标**: 实现 7 步 Wizard：欢迎+评估→状态展示→手工调整引导（step by step+可执行命令）→确认 baseline→选择部署模式→选择站点→完成
- **Acceptance**: 7 步流程可走完；Step 3 展示可执行命令（一键复制）；Step 4 语义化术语；进度条显示
- **依赖**: T3.1, T2.1
- **Ready When**: 共享组件+Store 层完成
- **初始队列状态**: pending
- **Selection Priority**: 5
- **Files / 触碰工件**: `src/app/wizard/page.tsx`, `src/components/wizard/*.tsx`
- **测试设计种子**: 7 步流程导航测试；Step 3 命令复制功能；Step 4 确认 baseline；Step 5 部署模式选择；Step 6 模板/手动添加
- **Verify**: `pnpm test -- --testPathPattern=wizard/`
- **预期证据**: Wizard 测试通过
- **完成条件**: 7 步引导流程完整可用

### T10.1: 通知系统 + 离线行为

- **目标**: 实现应用内通知（4 类型）+ 系统通知（Tauri API）+ 离线状态展示 + "查看全部通知"历史归档
- **Acceptance**: 通知推送正确展示；系统通知可用时同时推送；离线时展示上次缓存数据+"服务未运行"状态
- **依赖**: T2.1, T3.1
- **Ready When**: Store + 共享组件完成
- **初始队列状态**: pending
- **Selection Priority**: 6
- **Files / 触碰工件**: `src/components/shared/NotifBar.tsx`（扩展）, `src/hooks/use-offline.ts`
- **测试设计种子**: 4 种通知类型渲染；系统通知降级测试；离线状态→缓存数据展示；通知历史归档入口
- **Verify**: `pnpm test -- --testPathPattern=notification|offline`
- **预期证据**: 通知+离线测试通过
- **完成条件**: 通知系统和离线行为正确

### T10.2: 冷启动优化

- **目标**: 实现冷启动 3s 目标：Vite code splitting + 首页懒加载 + 缓存上次状态 + 延迟加载非关键模块
- **Acceptance**: 冷启动计时 ≤ 3s（从进程启动到首屏渲染）
- **依赖**: T4.1
- **Ready When**: 仪表盘页面完成
- **初始队列状态**: pending
- **Selection Priority**: 7
- **Files / 触碰工件**: `vite.config.ts`, `src/app/layout.tsx`, `src/app/dashboard/page.tsx`
- **测试设计种子**: Lighthouse 性能审计；首屏加载时间测量；代码分割验证（chunk 大小）
- **Verify**: 手动冷启动计时 + Vite bundle 分析
- **预期证据**: 冷启动 ≤ 3s
- **完成条件**: 冷启动性能达标

### T10.3: 跨平台一致性验证

- **目标**: Windows + WSL/Linux 截图对比验证 UI 一致性
- **Acceptance**: 关键页面（仪表盘/设置/诊断）在 Windows 和 WSL 下视觉一致
- **依赖**: T4.1, T7.1, T8.1
- **Ready When**: 核心页面完成
- **初始队列状态**: pending
- **Selection Priority**: 7
- **Files / 触碰工件**: 无新文件（验证性任务）
- **测试设计种子**: 关键页面截图对比；CSS 变量一致性检查
- **Verify**: 手动截图对比
- **预期证据**: 截图对比无显著差异
- **完成条件**: 跨平台 UI 一致

## 6. 依赖与关键路径

```
F001 T1.1 ─→ T1.1(前端结构)
F001~003 Commands ─→ T1.2(IPC) ─→ T2.1(Store) ─→ T3.1(共享组件)
T3.1 ─→ T4.1(仪表盘)
T3.1 ─→ T5.1(站点)
T3.1 ─→ T6.1(规则)
T3.1 ─→ T7.1(诊断)
T3.1 ─→ T8.1(设置)
T2.1a ─→ T2.1b(扩展Store)
T3.1 + T2.1a ─→ T9.1(Wizard)
T2.1a + T3.1 ─→ T10.1(通知+离线)
T4.1 ─→ T10.2(冷启动)
T4.1 + T7.1 + T8.1 ─→ T10.3(跨平台)
```

**关键路径**：F001 T1.1 → T1.1 → T1.2 → T2.1a → T3.1 → T4.1 → T10.2

**可并行任务组**：
- T4.1 + T5.1 + T6.1 + T7.1 + T8.1（均仅依赖 T3.1，可同时开始）
- T2.1b 可与 T3.1 并行（依赖 T2.1a）
- T9.1 + T10.1（依赖 T3.1 + T2.1a）

## 7. 完成定义与验证策略

| 里程碑 | DoD | 验证方式 |
|--------|-----|---------|
| M1 | 前端可导航 | `pnpm build` + 手动导航 |
| M2 | Store 状态管理正确 | `pnpm test stores/` |
| M3 | 共享组件可用 | `pnpm test components/shared/` |
| M4~M8 | 各页面功能完整 | 页面级组件测试 |
| M9 | 引导流程完整 | `pnpm test wizard/` |
| M10 | 非功能指标达标 | 手动验证+计时 |

## 8. 当前活跃任务选择规则

1. Feature 004 大部分任务依赖 F001~003 后端 Commands 完成
2. T1.1（前端结构）可在 F001 脚手架完成后立即开始
3. T1.2（IPC 封装）需要后端 Commands 接口定义完成（可使用 mock 并行）
4. **Current Active Task**: T1.1（等待 F001 脚手架完成后启动）

## 9. 任务队列投影视图

| 阶段 | 任务 | 状态 |
|------|------|------|
| Phase 1 | T1.1 前端结构 · T1.2 IPC 封装 | ⬜ pending（等待 F001） |
| Phase 2 | T2.1a 核心 Store · T2.1b 扩展 Store | ⬜ pending |
| Phase 3 | T3.1 共享组件 | ⬜ pending |
| Phase 4 | T4.1 仪表盘 · T5.1 站点 · T6.1 规则 · T7.1 诊断 · T8.1 设置 | ⬜ pending |
| Phase 5 | T9.1 Wizard · T10.1 通知+离线 | ⬜ pending |
| Phase 6 | T10.2 冷启动 · T10.3 跨平台 | ⬜ pending |

## 10. 风险与顺序说明

| 风险 | 影响 | 缓解 |
|------|------|------|
| WSL WebKitGTK 不可用 | T10.3 跨平台验证 | 检测并提示安装 |
| 冷启动超 3s | T10.2 | 首屏最小渲染+缓存+延迟加载 |
| 前端状态与后端不一致 | T2.1 | Tauri Event 推送+Store 细粒度更新 |
| 共享组件跨页面状态泄漏 | T3.1 | Zustand 无 Provider + 组件卸载清理 |
