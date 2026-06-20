# Feature 115: UX E2E 自动化测试基础设施正式化 — 需求规格

- **Feature**: 115-ux-e2e-infrastructure
- **阶段**: `hf-specify`
- **状态**: 草稿（待评审）
- **日期**: 2026-06-18
- **作者**: Teddy（项目管理者 & QA guardian）
- **上游输入**:
  - `features/114-ui-e2e-poc/poc-report.md` — PoC 可行性证据（QG1/2/3 全过）
  - `e2e/` — PoC 阶段产物（smoke + ipc 两 spec、wdio.conf.ts、独立 npm 包）
  - `docs/principles/testing-principles.md` — 三层测试方法论（宪法层）
  - `docs/principles/architectural-health-during-tdd.md` — TDD 纪律
  - `release/data/mihomo/config.yaml` — 开发态流量阻断根因

## 1. 概述

### 1.1 目的

F114 PoC 已证明 **tauri-driver + WebDriverIO + @wdio/tauri-service** 在 WSL2/Linux 下可建立可持续的桌面 E2E 基础设施。本 feature 将 PoC **正式化**：

1. **CI 集成**：把 e2e/ 套件接入 GitHub Actions Linux runner，作为 PR 合并门
2. **97s 优化**：实施 PoC 报告 §7 标记的 3 个优化杠杆，单次跑目标 ≤ 70s（节省 ≥ 25%）
3. **L4/L5 边界定义**：建立前端组件级（L4 vitest+RTL）与桌面端 E2E（L5 Tauri 完整链路）的测试分工矩阵
4. **基建规范化**：helpers / fixtures / 配置分层，为 F116+ 6 页面覆盖扩展做好承接

**不扩 spec 覆盖**：保留 F114 的 smoke + ipc 两 spec 作为基线，6 页面 E2E 推到 F116+ 立项。

### 1.2 范围

| 维度 | 入 | 出 |
|------|----|----|
| 测试 spec 数量 | smoke + ipc 共 2（沿用 F114） | 6 页面 E2E 覆盖（→ F116+） |
| CI 平台 | Linux（ubuntu-latest） | Windows / macOS（→ F116+ 或独立 feature） |
| 97s 优化 | 3 杠杆（cross-spec session / driver 常驻 / tauri-plugin-wdio 注册） | 缩短 waitforTimeout（评估后排除，稳定性优先） |
| 测试方法学 | L4/L5 边界定义矩阵 | L1/L2/L3 边界（已由 F113 闭合） |
| 工件侵入 | src-tauri/Cargo.toml 引入 tauri-plugin-wdio | 其它生产代码改动 |
| 主仓库侵入 | `pnpm test:e2e` 入口、`package.json` scripts | 其它根级配置 |
<!-- TODO id:05;status:close;date:2026-06-18T15:30 “测试 spec 数量”中，输出部分，不应该是“6 页面 E2E 覆盖（→ F116+）”，本feature负责smoke + ipc 共 2（沿用 F114）的feature测试，后续不会在F116+中实施整个项目的UX自动化测试，而是分散在后续的新的Feature开发过程中，本文档中与此相关的描述统一调整；任务处理结果：已在 spec.md（v2）§1.1 目的 / §1.2 范围表 / §7 Out of Scope 统一改为"不集中规划，由各 Feature 在自己 spec 阶段决定" -->

<!-- ? id:09;status:close;date:2026-06-18T15:30 经慎重考虑，在当前还没有首个稳定版本+单人开发的场景下，暂不引入CI/CD流程，不集成github的CI，保留本地直接push到github的main分支的现状，以此意见为准调整整个feature的范围及相关描述；任务处理结果：已在 spec.md（v2）删除 §2.1 CI 集成整节、SC-1（CI 合并门）、NFR-3.1.1 CI / NFR-3.1.3 cache、C-T2 workflow、R2/R4 CI 风险、§8 M3、§9 CI 验收证据；相关 FR 转为"本地基础设施 + 本地常驻"模式 -->

### 1.3 术语

| 术语 | 定义 |
|------|------|
| L4 | 前端组件级行为测试（vitest + React Testing Library，jsdom 环境） |
| L5 | 桌面端 E2E 测试（WebDriverIO + tauri-driver + 真实 GoGuo 二进制） |
| tauri-driver | Tauri 官方 WebDriver 桥（crates.io，v2.0.6） |
| @wdio/tauri-service | WebDriverIO 官方 Tauri 适配服务（npm，v1.1.0） |
| tauri-plugin-wdio | Tauri 插件，提供 window 状态查询 / mock 注入（@wdio/tauri-service 配套） |
| cross-spec session 复用 | wdio `restart: false` 配置，多 spec 文件复用同一 WebDriver session |
| driver 常驻 | CI workflow 中 tauri-driver 预启动，spec 直接连接而非自启 |
| **GoGuo 主仓库** | 仓库根 package.json 与 src-tauri/Cargo.toml 共同定义的工程主体 |
| **e2e/ 独立包** | `e2e/` 子目录独立 package.json，与主仓库 npm 依赖隔离 |

### 1.4 成功标准

| # | 标准 | 验证方式 |
|---|------|---------|
| SC-1 | CI Linux runner 跑 e2e/ 全套，作为 main 分支合并门 | `.github/workflows/e2e.yml` 存在，最近 5 次 PR 跑全过 |
| SC-2 | 单次 e2e 跑（smoke + ipc）耗时 ≤ 70s | CI 日志时间戳 + 本地 5 次连跑均值 |
| SC-3 | `tauri-plugin-wdio` 在 GoGuo src-tauri 注册，IPC retry warning 消失 | wdio 运行日志无 "Tauri plugin not available" 警告 |
| SC-4 | L4/L5 边界矩阵落盘，每个前端交互明确归属 | `docs/test-trace-matrix.md`（或独立文档）新增 L4/L5 列 |
| SC-5 | e2e/ 目录结构规范化（helpers / fixtures / specs 分层） | 目录结构符合 design.md 定义 |
| SC-6 | flakiness 在 CI 环境 ≤ 10%（10 次连跑） | CI workflow 触发 flakiness job 输出报告 |
<!-- TODO id:01;status:close;date:2026-06-18T15:30 关于SC-5成功标准，e2e/目录将作为本F115的端到端测试的目录结构，由于本Feature需要构建基础设施，那后续的Feature如F201进行e2e测试时，目录结构如何？需要统一规划项目级目录结构；任务处理结果：已在 spec.md（v2）§2.4 定义项目级统一目录约定 e2e/specs/<feature-id>/，F115 自身 spec 放 e2e/specs/f114-baseline/（沿用 PoC smoke + ipc），后续 F201 spec 放 e2e/specs/f201-*/；新增 §2.6 Feature 接入规范 -->

<!-- ? id:02;status:close;date:2026-06-18T15:30 后续开发时的Feature的e2e测试，与同一个wsl环境上生产环境的release程序运行中间，如何进行网络配置的隔离？这部分是否也需要纳入本Feature的范围？；任务处理结果：已纳入 F115 范围，spec.md（v2）新增 §2.5 网络/环境隔离方案，处理 GoGuo release 与 e2e 同 WSL 共存的 mihomo config 冲突（PoC 已发现该问题阻断开发态流量） -->

## 2. 功能需求

### 2.1 CI 集成（Linux）

#### FR-2.1.1 GitHub Actions workflow 建立

**要求**:
- FR-2.1.1-R1: 必须在 `.github/workflows/e2e.yml` 建立 e2e workflow，触发条件：PR 到 main、push 到 main、手动 workflow_dispatch
  - 验收测试: 文件存在，且在 PR 触发后能被 GitHub 识别
  - 可观测结果: PR 检查列表出现 "E2E (Linux)" 检查项
- FR-2.1.1-R2: workflow `jobs.e2e-linux` 必须使用 `ubuntu-latest`，步骤包含：(1) checkout (2) 装 Rust toolchain (3) 装 webkit2gtk-driver（apt）(4) 装 tauri-driver（cargo install）(5) `cargo build --release` (6) `pnpm install` (7) `cd e2e && pnpm install` (8) `GDK_BACKEND=x11 pnpm test`
  - 验收测试: workflow yaml 符合此步骤序列
  - 可观测结果: CI 日志依次出现各步骤
- FR-2.1.1-R3: workflow 必须配置为 **PR 合并门**（GitHub branch protection rule 引用此 check）
  - 验收测试: branch protection 配置引用 "E2E (Linux)"
  - 可观测结果: PR 在 e2e 失败时无法合并

#### FR-2.1.2 CI 资源约束

**要求**:
- FR-2.1.2-R1: workflow 必须 cache：`~/.cargo/registry`、`src-tauri/target`（基于 `Cargo.lock` hash）、`~/.cache/pnpm`、`e2e/node_modules`（基于 `e2e/package.json` hash）
  - 验收结果: 第二次跑开始 CI 时长显著下降
- FR-2.1.2-R2: workflow 必须 timeout 30 分钟（防卡死占用 runner）
- FR-2.1.2-R3: workflow 失败时必须上传 `wdio` 截图与日志作为 artifact（保留 14 天）

### 2.2 97s 优化杠杆实施

#### FR-2.2.1 cross-spec session 复用

**要求**:
- FR-2.2.1-R1: `e2e/wdio.conf.ts` 必须设置 `restart: false`，使 smoke.spec.ts 和 ipc.spec.ts 共享同一 WebDriver session
  - 验收测试: 配置项存在
  - 可观测结果: 单次跑日志只出现一次 "newSession"，GoGuo 进程只 spawn 一次
- FR-2.2.1-R2: 实施后双 spec 总耗时下降 ≥ 15s（对比 F114 baseline 95s）
  - 验收测试: 本地 5 次连跑均值 ≤ 80s

#### FR-2.2.2 tauri-driver 常驻（CI 模式）

**要求**:
- FR-2.2.2-R1: CI workflow 必须在 spec 跑之前预启动 tauri-driver（后台进程），e2e/wdio.conf.ts 通过 `TAURI_DRIVER_PORT` 环境变量复用已启 driver
  - 验收结果: spec 阶段日志不再出现 "启动 tauri-driver"，直接复用
- FR-2.2.2-R2: e2e/wdio.conf.ts 必须支持两种模式：自启（本地默认）+ 复用（CI，env `TAURI_DRIVER_REUSE=1`）
  - 验收结果: 本地 `pnpm test` 仍能自启，CI 设 env 后切换到复用模式
- FR-2.2.2-R3: 实施后 CI 总时长（不含 build）下降 ≥ 8s

#### FR-2.2.3 注册 tauri-plugin-wdio

**要求**:
- FR-2.2.3-R1: `src-tauri/Cargo.toml` 必须添加 `tauri-plugin-wdio` 依赖，版本与 @wdio/tauri-service v1.1.0 兼容
  - 验收测试: `cargo build --release` 成功
- FR-2.2.3-R2: `src-tauri/src/lib.rs` 必须在 tauri::Builder 链中注册 `.plugin(tauri_plugin_wdio::init())`
  - 验收测试: 编译通过，二进制能启动
- FR-2.2.3-R3: 实施后 wdio 运行日志不再出现 "Tauri plugin not available. Make sure @wdio/tauri-plugin is installed" 警告
- FR-2.2.3-R4: 必须验证 tauri-plugin-wdio 不影响 GoGuo 生产功能（已有的 625 测试全过、6 个前端页面手动冒烟通过）
  - 验收测试: `cargo test --workspace` 全过、`pnpm test` 全过
- FR-2.2.3-R5: 必须在 `docs/principles/testing-principles.md` 或 ADR 记录"生产 Cargo.toml 引入测试专用 plugin"的取舍（dev-only feature gate 评估 vs 直接引入）

#### FR-2.2.4 优化效果度量

**要求**:
- FR-2.2.4-R1: 必须在 `features/115-ux-e2e-infrastructure/design.md` 记录三项优化的预期与实测对比表（baseline 95s → 目标）
- FR-2.2.4-R2: 必须在 PoC report (`features/114-ui-e2e-poc/poc-report.md`) §7.1 立即项中标注"已在 F115 实施"

<!-- TODO id:03;status:close;date:2026-06-18T15:30 FR-2.2.4提供“验收测试”和“验收测试结果”，明确L1级测试要求；任务处理结果：已在 spec.md（v2）§2.2.4 增补 R3（L1 级单元测试：wdio.conf.ts 配置项解析逻辑、helper 函数边界条件、tauri-plugin-wdio 注册条件编译），明确验收测试函数命名约定与可观测结果 -->

### 2.3 L4/L5 测试分工边界

#### FR-2.3.1 边界矩阵文档

**要求**:
- FR-2.3.1-R1: 必须建立 L4/L5 测试分工矩阵，至少覆盖 6 个前端页面的代表性交互（dashboard / sites / rules / diagnostics / settings / wizard）
  - 验收测试: 文档存在，矩阵含 ≥ 30 条交互项
  - 可观测结果: 每条交互明确标注归属（L4 / L5 / 两者都）
- FR-2.3.1-R2: 矩阵必须包含 5 列：交互行为、L4 vitest 责任、L5 Tauri E2E 责任、依据、关联页面
- FR-2.3.1-R3: 矩阵必须明确以下"必 L5"原则：
  - 跨进程数据流（IPC → 后端 → 响应 → UI 更新）→ L5
  - Tauri 事件订阅与前端响应 → L5
  - Tauri webview 特性（X11/Wayland 切换、IPC 时序、WebKitGTK 渲染）→ L5
  - 跨页面状态同步（Zustand store 之外的可观测副作用）→ L5
<!-- TODO id:04;status:close;date:2026-06-18T15:30 FR-2.3.1中，明确对F201特性中涉及的所有UI界面，明确测试时要求的L4~L5等级要求，同时对其中的非UI能力，需要明确L1~L3等级要求，在本Feature中，需要明确基础设施如何参与后续特性的自动化测试，根据这个范围的重新调整，更新本文档相关内容。参见标注id:02；任务处理结果：已在 spec.md（v2）§2.3 重构为"测试等级矩阵"——以 F201 为首个应用案例，覆盖其全部 UI 界面（L4/L5）与非 UI 能力（L1/L2/L3）等级标注；§2.6 Feature 接入规范定义后续 Feature 如何按矩阵添加测试 -->

#### FR-2.3.2 边界执行约束

**要求**:
- FR-2.3.2-R1: 矩阵中标记为 L4 的交互不应在 e2e/ 重复（避免冗余）；标记为 L5 的交互必须有 e2e spec 承接（F116+ 实施时遵守）
- FR-2.3.2-R2: 必须在 `docs/principles/testing-principles.md` 增加"L4/L5 边界决策原则"小节，引用矩阵

### 2.4 基建规范化

#### FR-2.4.1 目录结构

**要求**:
- FR-2.4.1-R1: `e2e/` 目录必须按以下结构组织：
  ```
  e2e/
  ├── specs/           # 测试 spec（从 test/specs/ 迁移）
  ├── helpers/         # 复用 helper（invokeTauri、waitGoGuoReady 等）
  ├── fixtures/        # 测试数据 fixtures
  ├── wdio.conf.ts
  ├── package.json
  ├── tsconfig.json
  ├── .npmrc
  └── README.md
  ```
  - 验收测试: 目录结构符合，spec 跑通
- FR-2.4.1-R2: `helpers/` 必须抽取 PoC 阶段 inline 的 `invokeTauri<T>()` 函数为独立 helper（`helpers/tauri-ipc.ts`），spec 通过 import 引用

#### FR-2.4.2 主仓库入口

**要求**:
- FR-2.4.2-R1: 仓库根 `package.json` 必须添加 `"test:e2e": "cd e2e && pnpm test"` script
  - 验收测试: 根目录 `pnpm test:e2e` 能触发 e2e
- FR-2.4.2-R2: 仓库根 `package.json` 必须添加 `"test:all": "pnpm test && pnpm test:e2e"` 作为全套测试入口
<!-- ? id:06;status:close;date:2026-06-18T15:30 FR-2.4.2部分需要考虑后续其他feature开发时的自动化测试目录实施，确定是否对主仓库入口的设计是否有影响。参见标注id:05；任务处理结果：主仓库入口保留 pnpm test:e2e + pnpm test:all，但因 e2e/specs/ 按项目级 <feature-id>/ 分目录，入口命令不变，新增 spec 时无需改根 package.json；spec.md（v2）§2.4.2 已明确此设计 -->
## 3. 非功能需求

### 3.1 性能预算

- NFR-3.1.1: 单次 e2e 跑（smoke + ipc）在 CI Linux runner 上耗时 ≤ 70s（不含 build），baseline 95s
- NFR-3.1.2: 单次 e2e 跑在本地 WSL2 开发环境耗时 ≤ 80s（CI 与本地差异 ≤ 10s）
- NFR-3.1.3: CI cache 命中后总 workflow 时长（含 build + e2e）≤ 8 分钟
<!-- ? id:07;status:close;date:2026-06-18T15:30 NFR-3.1.3中，能否提升指标具备竞争力？；任务处理结果：因 id:09 撤 CI，NFR-3.1.3（CI cache 总 workflow 时长 ≤ 8 分钟）已作废，spec.md（v2）§3.1 已删除该条；本地性能预算保留 NFR-3.1.1（本地 ≤ 70s）作为竞争力指标 -->

### 3.2 稳定性

- NFR-3.2.1: CI 环境 10 次连跑 flakiness ≤ 10%（baseline PoC 0%，但 CI 环境变异性更高）
- NFR-3.2.2: 失败时必须产出可定位证据：截图、WebDriver session 日志、GoGuo 进程 stderr

### 3.3 可维护性

- NFR-3.3.1: spec 文件平均行数 ≤ 50 行（避免巨型 spec）
- NFR-3.3.2: helpers 抽取后，新增 spec 不应重复 inline `__TAURI_INTERNALS__.invoke` 逻辑
- NFR-3.3.3: 配置项（capabilities、driverProvider、waitforTimeout）必须在 wdio.conf.ts 内有注释说明取舍
<!-- ? id:08;status:close;date:2026-06-18T15:30 NFR-3.3.1中，建议扩大spec文件行数限制，否则spec文档无法承载一个feature端到端的业务；任务处理结果：spec.md（v2）§3.3 NFR-3.3.1 已从 ≤ 50 行扩展为 ≤ 200 行（单 feature 端到端业务可承载），并补充"超过 200 行须拆分 spec"约束 -->

### 3.4 兼容性

- NFR-3.4.1: tauri-plugin-wdio 引入后，已有的 625 个测试（455 后端 + 24 集成 + 146 前端）必须 100% 通过
- NFR-3.4.2: tauri-plugin-wdio 必须支持条件编译或 feature gate，评估是否仅在 test profile 启用（design.md 决策）

## 4. 约束

### 4.1 不变量（C-INvariants）

- C-I1: 不修改 F114 已验证的 PoC 关键配置：`browserName: "tauri"`、`tauri:options.application`、`GDK_BACKEND=x11`
- C-I2: 不引入新的 browserName（chrome/firefox 等）——仅 Tauri webview
- C-I3: 不删除 F114 的 smoke.spec.ts / ipc.spec.ts 内容（可重构结构，但测试覆盖不变）
- C-I4: 不改变 e2e/.npmrc 镜像隔离策略（避免污染主工程）

### 4.2 技术约束

- C-T1: tauri-plugin-wdio 版本必须与 @wdio/tauri-service v1.1.0 声明兼容的版本
- C-T2: CI workflow 必须用 GitHub Actions 官方 actions（actions/checkout、actions/cache、dtolnay/rust-toolchain 等）
- C-T3: 不引入 Selenium Grid 或其它外部 driver 编排服务
- C-T4: 不引入 Playwright、Cypress 或其它 E2E 框架（与 F114 选型一致）

### 4.3 流程约束

- C-P1: F115 spec 通过 review 后才能进入 design 阶段（interactive 模式）
- C-P2: 97s 优化每个杠杆必须先在本地验证效果再进 CI
- C-P3: tauri-plugin-wdio 引入必须先在 design.md 评估 dev-only feature gate 可行性

## 5. 风险登记

| # | 风险 | 等级 | 缓解 |
|---|------|------|------|
| R1 | tauri-plugin-wdio 引入后破坏 GoGuo 现有功能（插件副作用） | HIGH | FR-2.2.3-R4 全套回归；评估 dev-only feature gate（C-P3） |
| R2 | CI Linux runner 跑 WebKitWebDriver 不稳定（apt 依赖版本漂移） | MED | 在 workflow pin 关键依赖版本；定期跑 flakiness job 监控 |
| R3 | `restart: false` 导致 spec 间状态污染 | MED | helpers 提供 `beforeEach` 状态重置；spec 不依赖前序副作用 |
| R4 | driver 常驻模式下，CI worker 死锁或端口占用 | MED | workflow 在 job 结束显式 kill tauri-driver；超时保护 |
| R5 | L4/L5 矩阵定义过于严格，F116+ 实施时返工 | LOW | 矩阵作为"指南"而非"硬约束"，design 阶段允许调整 |
| R6 | mihomo config 在 CI 阻断 cargo/pnpm 流量（同 F114） | HIGH | CI 使用 npmmirror + rsproxy.cn 镜像（已验证） |

## 6. Open Questions

| # | 问题 | 决策时机 |
|---|------|---------|
| OQ-1 | tauri-plugin-wdio 是 dev-only feature gate 还是直接进生产 Cargo.toml？ | design 阶段（C-P3） |
| OQ-2 | L4/L5 边界矩阵放在 `docs/test-trace-matrix.md`（F113 产物）还是独立 `docs/l4-l5-boundary.md`？ | design 阶段 |
| OQ-3 | CI workflow 是否需要 nightly 跑 flakiness 监控 job（独立于 PR check）？ | design 阶段 |
| OQ-4 | 仓库根 `package.json` 是否需要添加 e2e/ 作为 pnpm workspace（统一 lockfile）？还是保持独立 npm 包？ | design 阶段 |
| OQ-5 | tauri-plugin-wdio 引入后是否影响 GoGuo 二进制体积 / 启动时间？需要量化对比 | tasks 实施时 |

## 7. Out of Scope（本 feature 不做）

| 项 | 推到 |
|----|------|
| 6 前端页面 E2E spec 编写（dashboard / sites / rules / diagnostics / settings / wizard） | F116+ |
| Windows / macOS CI 平台 | F116+ 或独立 feature |
| 视觉回归测试（@wdio/visual-service） | 后续 feature |
| 视频录制（wdio-video-reporter） | 后续 feature |
| Deep link / protocol handler 测试 | 后续 feature |
| Mock IPC 注入测试（@wdio/tauri-service mock 特性） | 后续 feature |
| 缩短 waitforTimeout（评估后排除，稳定性优先） | 不做 |

## 8. 里程碑（建议）

| 阶段 | 内容 | 依赖 |
|------|------|------|
| M1 design.md | 5 个 Open Questions 决策 + L4/L5 边界矩阵草稿 + tauri-plugin-wdio 引入方案 | spec.md 通过 review |
| M2 tasks.md | 拆解为可执行 task（CI workflow / 3 优化 / 目录重构 / 矩阵 / 文档） | design.md 通过 review |
| M3 CI workflow 落地 | `.github/workflows/e2e.yml` + branch protection 配置 | M2 |
| M4 优化实施 | 3 个优化杠杆逐项 RED-GREEN | M3 |
| M5 矩阵落盘 | L4/L5 边界文档 + testing-principles.md 更新 | M2 |
| M6 finalize | flakiness 验证 + PoC report 反向标注 + closeout | M3/M4/M5 |

## 9. 验收证据清单（finalize 前必须产出）

- [ ] `.github/workflows/e2e.yml` 在 main 分支最近 5 次 PR 全过
- [ ] 单次 e2e 跑 CI 耗时 ≤ 70s（截图证据）
- [ ] wdio 运行日志无 "Tauri plugin not available" 警告
- [ ] L4/L5 边界矩阵文档存在且 ≥ 30 条
- [ ] `cargo test --workspace && pnpm test` 全过（无回归）
- [ ] flakiness 10 次连跑报告 ≤ 10%
- [ ] F114 PoC report §7.1 立即项已标注"已实施"
- [ ] `docs/principles/testing-principles.md` 新增 L4/L5 决策原则小节
