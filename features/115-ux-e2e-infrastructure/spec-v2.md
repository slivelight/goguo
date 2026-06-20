# Feature 115: UX E2E 自动化测试基础设施正式化 — 需求规格

- **Feature**: 115-ux-e2e-infrastructure
- **阶段**: `hf-specify`
- **状态**: 草稿 v2（基于 v1 评审 9 项标注重构）
- **日期**: 2026-06-18
- **作者**: Teddy（项目管理者 & QA guardian）
- **版本历史**: `spec-v1.md`（保留 9 项评审标注作为历史）
- **上游输入**:
  - `features/114-ui-e2e-poc/poc-report.md` — PoC 可行性证据（QG1/2/3 全过）
  - `e2e/` — PoC 阶段产物（smoke + ipc 两 spec、wdio.conf.ts、独立 npm 包）
  - `docs/principles/testing-principles.md` — 三层测试方法论（宪法层）
  - `docs/principles/architectural-health-during-tdd.md` — TDD 纪律
  - `release/data/mihomo/config.yaml` — 开发态流量阻断根因
  - `features/201-first-run-baseline-confirm/spec.md` — 首个接入基础设施的下游 Feature（作为矩阵首案例）

## 1. 概述

### 1.1 目的

F114 PoC 已证明 **tauri-driver + WebDriverIO + @wdio/tauri-service** 在 WSL2/Linux 下可建立可持续的桌面 E2E 基础设施。本 feature 将 PoC **正式化为项目级基础设施**，核心交付：

1. **项目级 e2e 基础设施**：规范化 e2e/ 目录结构、helpers / fixtures 分层，可被任意后续 Feature 直接复用
2. **Feature 接入规范**：明确后续 Feature（如 F201）如何按规范添加 e2e spec，无需自行搭建
3. **97s 优化**：实施 PoC 报告 §7 标记的 3 个优化杠杆，单次跑目标 ≤ 70s（节省 ≥ 25%）
4. **测试等级矩阵**：建立 L1~L5 完整测试分工原则，以 **F201 为首个应用案例**覆盖其全部 UI/非 UI 能力的等级标注
5. **网络/环境隔离**：规范化处理 GoGuo release 与 e2e 测试在同一 WSL 环境共存时的网络冲突（PoC 已发现 mihomo config 阻断开发态流量）

**不集中规划 spec 覆盖**：本 feature 仅保留 F114 的 smoke + ipc 两 spec 作为基线（沿用），后续每个 Feature 在自己 spec 阶段决定是否新增 e2e 覆盖，不存在"F116+ 集中做 6 页面"的规划。

**不引入 CI/CD**：当前项目未发布稳定版本 + 单人开发，保留本地直推 main 的现状。CI 集成推到稳定版本之后再评估。

### 1.2 范围

| 维度 | 入 | 出 |
|------|----|----|
| 项目级 e2e 基础设施 | e2e/ 目录规范化、helpers/fixtures 分层、wdio.conf 优化 | 各 Feature 自建 e2e（已被基础设施承接） |
| Feature 接入规范 | 接入文档（§2.6）+ 矩阵 + helpers 复用契约 | 强制各 Feature 接入（建议性规范） |
| 97s 优化 | 3 杠杆（cross-spec session / driver 本地常驻 / tauri-plugin-wdio 注册） | 缩短 waitforTimeout（评估后排除，稳定性优先） |
| 测试等级矩阵 | F201 全部 UI/非 UI 能力的 L1~L5 标注 | F201 之外其它 Feature 的标注（各 Feature 自行） |
| 网络/环境隔离 | mihomo config 隔离方案 + dev/test profile | 改生产 mihomo config 行为 |
| 工件侵入 | src-tauri/Cargo.toml 引入 tauri-plugin-wdio | 其它生产代码改动 |
| 主仓库侵入 | `pnpm test:e2e` 入口、`package.json` scripts | 其它根级配置 |
| **CI/CD 集成** | **不集成**（撤，参见 v1 id:09） | 推到稳定版本后单独立项 |

### 1.3 术语

| 术语 | 定义 |
|------|------|
| L1 | Rust 单元测试（`#[cfg(test)]`，模块内 `mod tests`） |
| L2 | FR 验收测试（`src-tauri/tests/fr_acceptance/`，断言 spec FR 可观测结果） |
| L3 | 契约测试 + 管道集成测试（F113 建立） |
| L4 | 前端组件级行为测试（vitest + React Testing Library，jsdom 环境） |
| L5 | 桌面端 E2E 测试（WebDriverIO + tauri-driver + 真实 GoGuo 二进制） |
| tauri-driver | Tauri 官方 WebDriver 桥（crates.io，v2.0.6） |
| @wdio/tauri-service | WebDriverIO 官方 Tauri 适配服务（npm，v1.1.0） |
| tauri-plugin-wdio | Tauri 插件，提供 window 状态查询 / mock 注入（@wdio/tauri-service 配套） |
| cross-spec session 复用 | wdio `restart: false` 配置，多 spec 文件复用同一 WebDriver session |
| driver 本地常驻 | 开发者本地预启动 tauri-driver 后台进程，spec 通过 env `TAURI_DRIVER_REUSE=1` 复用而非自启 |
| **项目级 e2e 目录约定** | `e2e/specs/<feature-id>/<spec-name>.spec.ts` 的统一目录结构，所有 Feature 共用 `e2e/` 基础设施但 spec 按功能域分目录 |
| **GoGuo 主仓库** | 仓库根 package.json 与 src-tauri/Cargo.toml 共同定义的工程主体 |
| **e2e/ 独立包** | `e2e/` 子目录独立 package.json，与主仓库 npm 依赖隔离 |
| **接入规范** | 后续 Feature 在 e2e/ 添加新 spec 时必须遵守的目录命名、helper 复用、矩阵标注规则 |

### 1.4 成功标准

| # | 标准 | 验证方式 |
|---|------|---------|
| SC-1 | 项目级 e2e 基础设施规范化：e2e/ 目录结构、helpers、fixtures、wdio.conf 全部按 §2.4 定义落地 | 目录树 + 文件存在性检查 |
| SC-2 | 单次 e2e 跑（smoke + ipc）本地 WSL2 耗时 ≤ 70s（baseline 95s，节省 ≥ 25%） | 本地 5 次连跑均值，时间戳证据 |
| SC-3 | `tauri-plugin-wdio` 在 GoGuo src-tauri 注册，wdio 运行日志无 "Tauri plugin not available" 警告 | wdio 日志 grep |
| SC-4 | 测试等级矩阵文档落盘，**以 F201 为首个案例**：覆盖 F201 spec 全部 UI 界面（L4/L5 标注）+ 非 UI 能力（L1/L2/L3 标注） | 矩阵文档存在，F201 行数 ≥ F201 spec 的 FR 总数 × 0.6 |
| SC-5 | Feature 接入规范落盘：**AGENTS.md §7 强制条款** + `docs/principles/test-design-section-template.md` 章节模板 + `e2e/README.md` 接入流程；F201 作为首个按规范接入的 Feature 验证通过 | AGENTS.md / 模板文档 / e2e README 三处落盘；F201 design.md 含完整 §N 章节 |
| SC-6 | 网络/环境隔离方案落盘：dev/test/prod 三态 mihomo config 分离或运行时切换机制 | 隔离方案文档 + 本地验证：GoGuo release 运行 + e2e 跑同时存在不互相阻断 |
| SC-7 | 本地 10 次连跑 flakiness ≤ 10%（baseline PoC 0%） | 本地脚本 10 连跑报告 |
| SC-8 | tauri-plugin-wdio 引入后已有 625 个测试（455 后端 + 24 集成 + 146 前端）100% 通过 | `cargo test --workspace && pnpm test` 全过 |
| SC-9 | 单 Feature 测试入口可用：`pnpm test:feature -- f114-baseline` 与 `pnpm test:e2e:feature -- f114-baseline` 均跑通 | 命令执行成功，仅命中指定 feature 测试 |

## 2. 功能需求

### 2.1 项目级 e2e 基础设施规范化

#### FR-2.1.1 目录结构

**要求**:
- FR-2.1.1-R1: `e2e/` 目录必须按以下项目级结构组织：
  ```
  e2e/
  ├── specs/                       # 所有 Feature 的 spec 集中于此
  │   ├── f114-baseline/           # F114 PoC 沿用 spec（基础设施基线）
  │   │   ├── smoke.spec.ts
  │   │   └── ipc.spec.ts
  │   ├── f201-first-run/          # F201 接入示例（首个应用案例）
  │   │   └── *.spec.ts
  │   └── <feature-id>-<slug>/     # 后续 Feature 按此命名
  ├── helpers/                     # 跨 Feature 复用 helper
  │   ├── tauri-ipc.ts             # invokeTauri<T>() 等
  │   ├── wait.ts                  # waitForGoGuoReady 等
  │   └── env.ts                   # 环境检测（WSL/X11/tauri-driver 端口）
  ├── fixtures/                    # 跨 Feature 复用 fixtures
  │   └── sites.ts                 # 站点测试数据
  ├── wdio.conf.ts                 # 项目级 wdio 配置（specs glob 含 specs/**/*.spec.ts）
  ├── package.json
  ├── tsconfig.json
  ├── .npmrc                       # 镜像隔离
  └── README.md
  ```
  - 验收测试: `e2e/specs/f114-baseline/smoke.spec.ts` 与 `ipc.spec.ts` 从 PoC 路径迁移到位
  - 可观测结果: `pnpm test` 仍跑通 smoke + ipc（迁移无回归）
- FR-2.1.1-R2: 现有 PoC 路径 `e2e/test/specs/*.spec.ts` 必须迁移到 `e2e/specs/f114-baseline/`，`e2e/test/` 目录删除
  - 验收测试: `e2e/test/` 目录不存在
- FR-2.1.1-R3: 每个 Feature 的 spec 子目录命名必须是 `f<NNN>-<kebab-slug>`，slug 来自 features/<NNN>-<slug>/
  - 验收测试: 目录命名 grep 检查

#### FR-2.1.2 helpers 抽取

**要求**:
- FR-2.1.2-R1: PoC 阶段 inline 在 ipc.spec.ts 中的 `invokeTauri<T>()` 函数必须抽取到 `e2e/helpers/tauri-ipc.ts`，spec 通过 `import { invokeTauri } from "../../helpers/tauri-ipc"` 引用
  - 验收测试: ipc.spec.ts 无 inline `__TAURI_INTERNALS__` 调用
- FR-2.1.2-R2: 必须新增 `e2e/helpers/wait.ts`，提供 `waitForGoGuoReady(timeout?: number)` helper（封装 PoC 的 `$("body").waitForExist` 模式）
- FR-2.1.2-R3: 必须新增 `e2e/helpers/env.ts`，提供：
  - `isWSL(): boolean` — 检测 WSL 环境
  - `ensureX11Backend(): void` — 强制 GDK_BACKEND=x11（F111 教训）
  - `getTauriDriverPort(): number` — 从 env 读取或默认 4444
  - `shouldReuseDriver(): boolean` — 检测 `TAURI_DRIVER_REUSE=1`

#### FR-2.1.3 主仓库入口

**要求**:
- FR-2.1.3-R1: 仓库根 `package.json` 必须添加 `"test:e2e": "cd e2e && pnpm test"` script
  - 验收测试: 根目录 `pnpm test:e2e` 能触发 e2e
- FR-2.1.3-R2: 仓库根 `package.json` 必须添加 `"test:all": "pnpm test && pnpm test:e2e"` 作为全套测试入口
- FR-2.1.3-R3: 入口设计必须保持稳定：后续 Feature 新增 spec 进 `e2e/specs/<feature-id>/` 时**无需修改**根 package.json
  - 验收测试: 模拟 F201 添加 spec 文件，根 package.json 无 diff
- FR-2.1.3-R4: 必须提供**单 Feature 全量测试**入口 `pnpm test:feature -- <feature-id>`（如 `pnpm test:feature -- f201`），按 feature-id 过滤以下三层：
  - **L1+L2+L3 后端测试**: `cargo test --workspace <feature-id>`（依赖 F113 已建立的命名约定，FR 验收测试模块按 `f<NNN>_<slug>.rs` 命名）
  - **L4 前端测试**: `pnpm test -- <feature-id>`（vitest `-t` filter，spec 文件须含 feature-id 标签或路径段）
  - 验收测试: `pnpm test:feature -- f114` 跑通且仅执行 f114 相关测试
  - 可观测结果: 输出汇总：3 层各自的通过/失败数
- FR-2.1.3-R5: 必须提供**单 Feature e2e 测试**入口 `pnpm test:e2e:feature -- <feature-id>`，等价于 `cd e2e && pnpm test -- --spec specs/<feature-id>/**/*.spec.ts`
  - 验收测试: `pnpm test:e2e:feature -- f114-baseline` 仅跑 smoke + ipc
  - 可观测结果: 仅执行指定 feature-id 子目录下的 spec
- FR-2.1.3-R6: 单 Feature 入口必须在 e2e/package.json 与根 package.json 双层提供（根入口转发到 e2e/）
- FR-2.1.3-R7: 单 Feature 入口的 feature-id 参数必须支持 tab 自动补全（扫描 `e2e/specs/` 与 `features/` 目录）——若实现成本高，可降级为 README 列出可用 id 清单
  - 验收测试: `pnpm test:feature -- <Tab>` 列出至少 f114-baseline
<!-- ? id:01;status:close;date:2026-06-18T16:10 明确一下：在仓库根目录下，除了全量测试，e2e测试入口外，是否有针对单一feature的全量测试，和e2e测试入口？；任务处理结果：已在 FR-2.1.3 增补 R4（pnpm test:feature 三层过滤：cargo test/vitest by feature-id）、R5（pnpm test:e2e:feature 按 e2e/specs/<id>/ 过滤）、R6（双层入口）、R7（feature-id 自动补全或清单），覆盖单 Feature 的全量测试与 e2e 入口 -->

### 2.2 97s 优化杠杆实施

#### FR-2.2.1 cross-spec session 复用

**要求**:
- FR-2.2.1-R1: `e2e/wdio.conf.ts` 必须设置 `restart: false`，使多个 spec 文件共享同一 WebDriver session
  - 验收测试: 配置项存在
  - 可观测结果: 单次跑日志只出现一次 "newSession"，GoGuo 进程只 spawn 一次
- FR-2.2.1-R2: 实施后 smoke + ipc 总耗时下降 ≥ 15s（对比 F114 baseline 95s）
  - 验收测试: 本地 5 次连跑均值 ≤ 80s
- FR-2.2.1-R3: spec 间状态污染防护：每个 spec 必须 `beforeEach` 调用 `resetGoGuoState()` helper（清理 SiteRulesState 等可变状态）
  - 验收测试: helper 存在，spec 引用

#### FR-2.2.2 tauri-driver 本地常驻

**要求**:
- FR-2.2.2-R1: e2e/wdio.conf.ts 必须支持两种模式：
  - **自启模式**（默认）：`TAURI_DRIVER_REUSE` 未设或为 0，wdio 通过 @wdio/tauri-service 自动 spawn tauri-driver
  - **复用模式**：`TAURI_DRIVER_REUSE=1`，wdio 跳过 spawn，直接连接 env 指定的 `TAURI_DRIVER_PORT`（默认 4444）
  - 验收测试: 两种模式下 `pnpm test` 均能跑通
- FR-2.2.2-R2: 必须提供本地常驻启动脚本 `e2e/scripts/start-driver.sh`，开发者本地预启 tauri-driver 用
  - 验收测试: 脚本执行后 `ss -ltn | grep 4444` 显示端口监听
- FR-2.2.2-R3: 实施后复用模式下单次跑耗时下降 ≥ 8s（对比自启模式）
  - 验收测试: 两种模式各跑 5 次均值对比
- FR-2.2.2-R4: README 必须说明两种模式的使用场景：日常开发用复用模式（快）、首次/CI 验证用自启模式（隔离）

#### FR-2.2.3 注册 tauri-plugin-wdio

**要求**:
- FR-2.2.3-R1: `src-tauri/Cargo.toml` 必须添加 `tauri-plugin-wdio` 依赖，版本与 @wdio/tauri-service v1.1.0 声明兼容
  - 验收测试: `cargo build --release` 成功
- FR-2.2.3-R2: `src-tauri/src/lib.rs` 必须在 tauri::Builder 链中注册 `.plugin(tauri_plugin_wdio::init())`
  - 验收测试: 编译通过，二进制能启动
- FR-2.2.3-R3: 实施后 wdio 运行日志不再出现 "Tauri plugin not available. Make sure @wdio/tauri-plugin is installed" 警告
- FR-2.2.3-R4: 必须验证 tauri-plugin-wdio 不影响 GoGuo 生产功能：已有 625 测试全过、6 个前端页面手动冒烟通过
  - 验收测试: `cargo test --workspace` 全过、`pnpm test` 全过
- FR-2.2.3-R5: 必须在 design.md 评估 dev-only feature gate 可行性，决策"是否仅 test profile 启用"或"全 profile 启用"
- FR-2.2.3-R6: 若选择全 profile 启用，必须在 `docs/principles/testing-principles.md` 或 ADR 记录"生产 Cargo.toml 引入测试专用 plugin"的取舍

#### FR-2.2.4 优化效果度量（含 L1 验收测试）

**要求**:
- FR-2.2.4-R1: 必须在 `features/115-ux-e2e-infrastructure/design.md` 记录三项优化的预期与实测对比表（baseline 95s → 目标 ≤ 70s）
- FR-2.2.4-R2: 必须在 PoC report (`features/114-ui-e2e-poc/poc-report.md`) §7.1 立即项中标注"已在 F115 实施"
- FR-2.2.4-R3 (L1 验收测试): 必须为优化逻辑本身提供 Rust/TS 单元测试（L1 级）：
  - `e2e/helpers/env.ts` 的 `shouldReuseDriver()` / `getTauriDriverPort()` 边界条件（env 未设/空字符串/非法端口）
  - `e2e/wdio.conf.ts` 中 capabilities 构造逻辑（独立函数化后单测）
  - `src-tauri/src/lib.rs` 中 tauri-plugin-wdio 注册的条件编译分支（若用 feature gate）
  - 验收测试: `pnpm --filter e2e test:unit` 与 `cargo test --lib` 通过
  - 可观测结果: 单测覆盖率报告显示上述函数 / 分支 ≥ 80%

### 2.3 测试等级矩阵（L1~L5）

#### FR-2.3.1 矩阵文档建立

**要求**:
- FR-2.3.1-R1: 必须建立测试等级矩阵文档（位置由 OQ-1 决策），文档名暂定 `docs/test-level-matrix.md`
- FR-2.3.1-R2: 矩阵必须覆盖 GoGuo 全部能力（不限于 UI），每条记录包含 6 列：
  - **能力 ID**（如 F201-FR-1.1-eval-trigger）
  - **能力描述**（一句话）
  - **关联 Feature / FR ID**
  - **L1 责任**（Rust 单测，标注函数名或 "N/A"）
  - **L2 责任**（FR 验收测试，标注测试函数名或 "N/A"）
  - **L3 责任**（契约/管道测试或 "N/A"）
  - **L4 责任**（vitest+RTL 测试名或 "N/A"）
  - **L5 责任**（e2e spec 文件:测试名或 "N/A"）
  - **依据**（为何这样分工，一句话）
- FR-2.3.1-R3: 矩阵首案例必须完整覆盖 **F201 spec 全部 FR**：
  - F201 spec 中的全部 UI 界面 → 必须明确 L4 或 L5 等级
  - F201 spec 中的全部非 UI 能力（后端命令、状态管理、数据流）→ 必须明确 L1/L2/L3 等级
  - 验收测试: 矩阵 F201 行数 ≥ F201 spec FR 总数 × 0.6
- FR-2.3.1-R3a (两阶段完整性，回答"何时矩阵算完整"):
  - **阶段 1（F115 完成时，矩阵结构完整）**: F201 的所有 FR 已在矩阵中占行，每行的 L1~L5 责任列**至少有等级标注**（L1/L2/L3/L4/L5/N/A）；测试函数名列允许为空（标注 `<TBD by F201 design>`）
  - **阶段 2（F201 finalize 时，测试代码完整）**: F201 design 阶段填入测试函数名（FR-2.4.3），F201 TDD/finalize 阶段所有声明的测试代码均已实现且通过
  - 验收测试: F115 finalize 时矩阵满足阶段 1；F201 finalize 时矩阵满足阶段 2
  - 可观测结果: 阶段 1 → 矩阵 grep `<TBD` 计数 ≤ (F201 行数 × 0.4)；阶段 2 → 矩阵 grep `<TBD` 计数 = 0
- FR-2.3.1-R4: 矩阵必须明确以下"等级决策原则"：
  - 跨进程数据流（IPC → 后端 → 响应 → UI 更新）→ **必须 L5**
  - Tauri 事件订阅与前端响应 → **必须 L5**
  - Tauri webview 特性（X11/Wayland 切换、IPC 时序、WebKitGTK 渲染）→ **必须 L5**
  - 跨页面状态同步（Zustand store 之外的可观测副作用）→ **必须 L5**
  - 单组件渲染、props 处理、内部状态机 → **L4**
  - 单 Rust 模块纯函数、数据结构 → **L1**
  - Rust trait 一致性、DTO 往返 → **L3**
  - FR 级可观测行为（不依赖 UI） → **L2**
- FR-2.3.1-R5: 矩阵必须明确"基础设施如何参与后续 Feature 测试"——通过 §2.4 接入规范，**贯穿 HF 全流程**（specify → design → tasks → test-driven-dev → finalize），每个 Feature 在 design 阶段必填"L1~L5 自动化测试设计"章节（FR-2.4.4），并在编码启动前完成测试用例 + 数据 + 脚本设计。
<!-- ? id:02;status:close;date:2026-06-18T16:10 明确有了FR-2.3.1-R3后，是否在开发完F201后，能实施本Feature中准备好的完整的测试矩阵？；任务处理结果：新增 FR-2.3.1-R3a 明确两阶段完整性——F115 完成时矩阵"结构完整"（所有 F201 FR 有行 + 等级标注，函数名允许 TBD），F201 finalize 时矩阵"测试代码完整"（所有函数名填齐、测试通过） -->

<!-- TODO id:03;status:close;date:2026-06-18T16:10 FR-2.3.1-R5中，需要明确该接入规范如何引入整个HF框架，支持后续每个Feature的自动化测试，包含UX和非用户交互界面的测试，最好在项目根目录下的AGENTS.md文档中明确强制性规范，参考下面的FR-2.3.2说明。在我的理解中，对于每个feature，先确定spec，再输出design（HF框架中，有单独的UI设计节点），在design中，要专门有个章节，完成L1~L5自动化测试的设计，测试用例+数据+脚本的设计，需要在编码启动之前，在详细设计中一并输出；任务处理结果：已重构 FR-2.3.1-R5（贯穿 HF 全流程）+ 新增 FR-2.4.3（AGENTS.md 强制规范）+ FR-2.4.4（design.md "L1~L5 自动化测试设计"章节模板）+ FR-2.4.5（HF 各阶段检查点），覆盖 UX 与非 UX 测试，强制编码前完成测试设计 -->

#### FR-2.3.2 矩阵执行约束

**要求**:
- FR-2.3.2-R1: 矩阵中标记为 L4 的能力不应在 `e2e/` 重复（避免冗余）；标记为 L5 的能力必须有 e2e spec 承接
- FR-2.3.2-R2: 必须在 `docs/principles/testing-principles.md` 增加"L1~L5 等级决策原则"小节，引用矩阵
- FR-2.3.2-R3: 后续 Feature review 必须检查矩阵更新（hf-design / hf-finalize 阶段加入检查点）

### 2.4 Feature 接入规范

#### FR-2.4.1 接入流程文档

**要求**:
- FR-2.4.1-R1: 必须在 `e2e/README.md` 增加"Feature 接入流程"章节，覆盖：
  - **Step 1**: 在 `e2e/specs/f<NNN>-<slug>/` 创建目录
  - **Step 2**: 复用 `e2e/helpers/` 中的 helper，不 inline `__TAURI_INTERNALS__`
  - **Step 3**: 在 `docs/test-level-matrix.md` 为本 Feature 的每个 FR 添加 L1~L5 责任行
  - **Step 4**: spec 命名约定：`<scenario>-<action>.spec.ts`（如 `dashboard-eval-trigger.spec.ts`）
  - **Step 5**: 每个 spec 必须以 `describe("<Feature ID>: <scenario>")` 开头，便于矩阵追溯
- FR-2.4.1-R2: F201 必须作为首个按规范接入的 Feature 验证：
  - 在 F201 design/tasks 阶段引用本规范
  - 至少 1 个 F201 e2e spec 落地（spec 内容由 F201 决定，F115 仅验证流程）
  - 矩阵中 F201 行齐全
  - 验收测试: `e2e/specs/f201-first-run/` 至少 1 个 spec，矩阵 F201 行齐全

#### FR-2.4.2 接入规范可执行性

**要求**:
- FR-2.4.2-R1: 必须提供脚本或 lint 规则验证接入规范：
  - 所有 spec 在 `e2e/specs/<f\d{3}-slug>/` 目录下
  - 所有 spec 顶部 `describe` 含 Feature ID
  - 所有 helper import 来自 `e2e/helpers/`
  - 验收测试: `pnpm --filter e2e lint` 或独立脚本通过

#### FR-2.4.3 AGENTS.md 强制规范

**要求**:
- FR-2.4.3-R1: 项目根 `AGENTS.md` 必须新增章节 **"§7. Feature 自动化测试设计强制规范"**（编号可由现有结构决定，作为 §6 后续），声明以下硬约束（适用于所有未来 Feature，F115 自身豁免以避免循环依赖）：
  - **强制条款 1**: 每个 Feature 在 `hf-design` 阶段必须输出一节"§X L1~L5 自动化测试设计"（章节位置由 §2.4.4 模板定义），作为 design.md 的必填章节
  - **强制条款 2**: 该章节未通过 review **不得进入** `hf-tasks` 阶段（即编码启动前必须完成测试设计）
  - **强制条款 3**: 章节必须同时覆盖 **UX 能力（L4/L5）**与**非 UI 能力（L1/L2/L3）**，对每条 FR 给出测试用例 + 数据 + 脚本入口
  - **强制条款 4**: `hf-finalize` 阶段必须验证该章节中所有声明的测试均已实现且通过，否则不通过完成门
- FR-2.4.3-R2: `AGENTS.md` §4 "Coding / Testing / Architecture 标准"必须交叉引用 §7 与 `docs/test-level-matrix.md`、`docs/principles/testing-principles.md`
- FR-2.4.3-R3: 仓库根 `README.md` "Active feature 指针来源"附近必须新增一句"所有 Feature design 必须含 L1~L5 自动化测试设计章节（详见 AGENTS.md §7）"
- FR-2.4.3-R4: AGENTS.md 改动必须作为 F115 tasks 阶段交付物（spec 阶段只定义"要改什么"），实际 commit message 遵循 `docs(agents): enforce L1~L5 test design section per feature`
  - 验收测试: `grep -n "Feature 自动化测试设计强制规范" AGENTS.md` 返回非空
  - 可观测结果: AGENTS.md 含 §7 完整章节，含 4 条强制条款

#### FR-2.4.4 design.md "L1~L5 自动化测试设计"章节模板

**要求**:
- FR-2.4.4-R1: F115 自身的 `design.md` 必须定义并应用一个**章节模板**（同时作为后续 Feature 的参照），章节标题为 `## N. L1~L5 自动化测试设计`，子节结构如下：

  ```markdown
  ## N. L1~L5 自动化测试设计

  > 强制章节（AGENTS.md §7）。本章在编码启动前完成，覆盖本 Feature 全部 FR。

  ### N.1 测试等级矩阵填充
  - 列出本 Feature 在 docs/test-level-matrix.md 中新增的行（至少含 FR ID + 等级标注）
  - 引用 F115 spec FR-2.3.1-R4 决策原则

  ### N.2 测试用例设计（逐层）
  #### N.2.1 L1（Rust 单元测试）
  | 测试函数 | 模块 | 断言 | 覆盖率目标 |
  |---------|------|------|----------|
  | ... | ... | ... | ... |

  #### N.2.2 L2（FR 验收测试）
  | 测试函数 | 文件路径 | 可观测结果 | 关联 FR |
  |---------|---------|----------|--------|
  | ... | ... | ... | ... |

  #### N.2.3 L3（契约 / 管道集成测试）
  | 测试函数 | 类型（契约/管道）| 关键断言 | 关联 FR |
  |---------|----------------|---------|--------|
  | ... | ... | ... | ... |

  #### N.2.4 L4（vitest + RTL）
  | spec 文件 | describe/it | 渲染场景 | 关联 FR |
  |---------|-----------|---------|--------|
  | ... | ... | ... | ... |

  #### N.2.5 L5（e2e spec）
  | spec 文件 | e2e/specs/<feature-id>/... | describe/it | 关联 FR |
  |---------|-----------------------------|-----------|--------|
  | ... | ... | ... | ... |

  ### N.3 测试数据
  - 共享 fixtures（e2e/fixtures/）vs Feature 私有 fixtures（features/<NNN>/fixtures/）
  - 测试用 site_id / mock 数据清单

  ### N.4 测试脚本入口
  - 单 Feature 全量测试: `pnpm test:feature -- <id>`
  - 单 Feature e2e: `pnpm test:e2e:feature -- <id>`
  - 全套: `pnpm test:all`

  ### N.5 TDD 执行顺序
  - 列出 RED → GREEN → REFACTOR 的实施顺序（按 FR 优先级）
  ```

- FR-2.4.4-R2: 模板必须存放于 `docs/principles/test-design-section-template.md`（宪法层），供所有 Feature 引用
  - 验收测试: 文件存在；F115 design.md 含完整 §N 章节
- FR-2.4.4-R3: F201 design 阶段必须使用此模板（作为接入规范的首个应用案例，参见 FR-2.4.1-R2）

#### FR-2.4.5 HF 全流程检查点

**要求**:
- FR-2.4.5-R1: 必须明确 HF 各阶段对本接入规范的检查点（落在 `docs/principles/test-design-section-template.md` 或 AGENTS.md §7）：

  | HF 阶段 | 检查点 | 责任人 |
  |--------|-------|-------|
  | `hf-specify` | 矩阵为本 Feature 占行（FR ID + 等级标注，函数名允许 TBD） | spec 作者 |
  | `hf-design` | design.md 含完整 §N "L1~L5 自动化测试设计"章节；矩阵函数名填齐；测试用例 + 数据 + 脚本设计完成 | design 作者 |
  | `hf-tasks` | tasks.md 拆解时含每条测试的实施 task（按 §N.5 顺序） | tasks 作者 |
  | `hf-test-driven-dev` | 按 §N.5 RED-GREEN 执行；不允许跳过 L1~L5 任意层 | 实施者 |
  | `hf-finalize` | 验证 §N 中所有声明的测试已实现且通过；矩阵 TBD 计数 = 0 | finalize 审查 |

- FR-2.4.5-R2: F115 自身的 design.md 必须按 §N 模板填写（作为模板自验证）
  - 验收测试: F115 design.md 含 §N 全部子节，且 §N.2 各表非空（至少 1 行 L1 + 1 行 L5）
  - 可观测结果: F115 自身的测试设计可被其它 Feature 直接参照

### 2.5 网络/环境隔离方案

#### FR-2.5.1 问题边界

**要求**:
- FR-2.5.1-R1: 必须在 design.md 明确问题边界——GoGuo release 二进制启动时会写 `/etc/environment` 注入代理环境变量（基于 `release/data/mihomo/config.yaml`），同时 mihomo config 的规则阻断开发态 cargo/pnpm 流量（PoC 已发现），导致同一 WSL 环境下 e2e 测试与开发活动互相干扰
- FR-2.5.1-R2: 必须明确三种场景的隔离需求：
  - **场景 A**: GoGuo release 运行 + 开发活动（cargo/pnpm）共存 — 需开发态流量不被 mihomo 阻断
  - **场景 B**: GoGuo release 运行 + e2e 测试共存 — e2e 调起 release 二进制时不破坏开发者当前 mihomo 状态
  - **场景 C**: 多个 GoGuo 实例（开发调试 + e2e）共存 — 端口冲突、状态文件冲突

#### FR-2.5.2 隔离方案设计

**要求**:
- FR-2.5.2-R1: design.md 必须给出至少一个可行方案，候选包括（不限于）：
  - **方案 P1**: mihomo config 拆分为 dev/test/prod 三态，按 env `GOGUO_PROFILE` 切换
  - **方案 P2**: 在现有 mihomo config 增加 `site-dev-tools` ruleset 放行 cargo/pnpm 流量
  - **方案 P3**: e2e 测试启动 GoGuo 时使用独立 HOME 与独立 config 路径（OS 标准应用目录隔离）
  - **方案 P4**: 容器化 e2e（docker/podman）完全隔离
- FR-2.5.2-R2: 所选方案必须在 design.md 评估：实施成本、对生产用户影响、对开发流程侵入、可逆性
- FR-2.5.2-R3: 所选方案实施后必须本地验证：GoGuo release 运行 + e2e 跑 + cargo build 三者并发不互相阻断
  - 验收测试: 三并发场景的步骤脚本跑通
  - 可观测结果: cargo build 流量正常、e2e 通过、GoGuo release 网络功能正常

### 2.6 配置与文档同步

#### FR-2.6.1 配置文件同步

**要求**:
- FR-2.6.1-R1: 所有新增配置（如 `TAURI_DRIVER_REUSE`、`GOGUO_PROFILE`、wdio spec glob 等）必须在 `e2e/README.md` 列出 env 表
- FR-2.6.1-R2: 仓库根 `.gitignore` 必须确认 `e2e/node_modules`、`e2e/.wdio-cache`（如有）被忽略

#### FR-2.6.2 文档同步

**要求**:
- FR-2.6.2-R1: `docs/principles/testing-principles.md` 必须新增"L1~L5 等级决策原则"小节，引用 `docs/test-level-matrix.md`
- FR-2.6.2-R2: `features/114-ui-e2e-poc/poc-report.md` §7.1 立即项必须标注"已在 F115 实施"
- FR-2.6.2-R3: 仓库根 `README.md` 测试入口必须更新为 `pnpm test:all`（含 e2e）

## 3. 非功能需求

### 3.1 性能预算

- NFR-3.1.1: 单次 e2e 跑（smoke + ipc）在本地 WSL2 开发环境耗时 ≤ 70s（baseline F114 PoC 实测 95s），节省 ≥ 25%
- NFR-3.1.2: driver 复用模式（`TAURI_DRIVER_REUSE=1`）下单次跑比自启模式快 ≥ 8s
- NFR-3.1.3: helpers 抽取后 spec 平均行数比 PoC inline 时期下降 ≥ 30%（基线：ipc.spec.ts 60 行 → 目标 ≤ 42 行）

### 3.2 稳定性

- NFR-3.2.1: 本地 WSL2 环境 10 次连跑 flakiness ≤ 10%（baseline PoC 0%）
- NFR-3.2.2: 失败时必须产出可定位证据：截图（`e2e/.wdio-screenshots/`）、WebDriver session 日志、GoGuo 进程 stderr
- NFR-3.2.3: `restart: false` 实施后必须保证 spec 间状态隔离（每个 spec `beforeEach` 重置）

### 3.3 可维护性

- NFR-3.3.1: spec 文件平均行数 ≤ 200 行（一个 Feature 端到端业务可承载）；超过 200 行必须按场景拆分为多 spec
- NFR-3.3.2: helpers 抽取后，新增 spec 不应重复 inline `__TAURI_INTERNALS__.invoke` 逻辑
- NFR-3.3.3: 配置项（capabilities、driverProvider、waitforTimeout、`TAURI_DRIVER_REUSE`）必须在 wdio.conf.ts 内有注释说明取舍
- NFR-3.3.4: 接入规范必须有可执行 lint 或脚本验证（FR-2.4.2）

### 3.4 兼容性

- NFR-3.4.1: tauri-plugin-wdio 引入后已有的 625 个测试（455 后端 + 24 集成 + 146 前端）必须 100% 通过
- NFR-3.4.2: 网络/环境隔离方案必须不影响 GoGuo release 的生产用户行为（不动 release/data/mihomo/config.yaml 的生产规则，或新增 dev/test profile 时生产 profile 不变）

## 4. 约束

### 4.1 不变量（C-Invariants）

- C-I1: 不修改 F114 已验证的 PoC 关键配置：`browserName: "tauri"`、`tauri:options.application`、`GDK_BACKEND=x11`
- C-I2: 不引入新的 browserName（chrome/firefox 等）——仅 Tauri webview
- C-I3: 不删除 F114 的 smoke.spec.ts / ipc.spec.ts 测试覆盖（可迁移路径与重构结构，但测试用例与断言不变）
- C-I4: 不改变 e2e/.npmrc 镜像隔离策略（避免污染主工程）
- C-I5: 不修改 `release/data/mihomo/config.yaml` 现有生产规则（如需调整走 dev/test profile 或独立方案）

### 4.2 技术约束

- C-T1: tauri-plugin-wdio 版本必须与 @wdio/tauri-service v1.1.0 声明兼容的版本
- C-T2: 不引入 Selenium Grid 或其它外部 driver 编排服务
- C-T3: 不引入 Playwright、Cypress 或其它 E2E 框架（与 F114 选型一致）
- C-T4: 网络/环境隔离方案不引入 Docker/Podman（除非 OQ-4 决策为容器化）
- C-T5: 不引入 GitHub Actions 或其它 CI 平台（撤 CI，v1 id:09）

### 4.3 流程约束

- C-P1: F115 spec 通过 review 后才能进入 design 阶段（interactive 模式）
- C-P2: 97s 优化每个杠杆必须先在本地验证效果再合入 main
- C-P3: tauri-plugin-wdio 引入必须先在 design.md 评估 dev-only feature gate 可行性
- C-P4: F201 接入规范验证（FR-2.4.1-R2）必须在 F201 design 阶段而非 F115 内完成（F115 仅提供规范与基础设施，不替 F201 写 spec）

## 5. 风险登记

| # | 风险 | 等级 | 缓解 |
|---|------|------|------|
| R1 | tauri-plugin-wdio 引入后破坏 GoGuo 现有功能（插件副作用） | HIGH | FR-2.2.3-R4 全套回归；评估 dev-only feature gate（C-P3） |
| R2 | `restart: false` 导致 spec 间状态污染 | MED | helpers 提供 `resetGoGuoState()`；spec `beforeEach` 重置；NFR-3.2.3 验证 |
| R3 | driver 复用模式下端口占用或僵尸进程 | MED | 复用脚本提供 stop-driver.sh；超时保护 |
| R4 | L1~L5 矩阵定义过于严格，后续 Feature 接入返工 | MED | 矩阵作为"指南"而非"硬约束"，design 阶段允许调整；FR-2.3.1-R4 决策原则可演进 |
| R5 | 网络/环境隔离方案选型错误（P1~P4）导致返工 | MED | FR-2.5.2-R2 多维评估；优先选可逆方案（P2/P3） |
| R6 | mihomo config 阻断开发态 cargo/pnpm 流量持续影响开发（同 F114） | HIGH | FR-2.5 网络隔离方案直接处理；e2e/.npmrc 与 ~/.cargo/config.toml 镜像配置已验证绕过 |
| R7 | F201 作为接入规范首个案例验证失败，规范需大改 | LOW | F201 验证（FR-2.4.1-R2）作为 F115 finalize 前置条件；早发现早调整 |
| R8 | spec-v1 → spec-v2 大范围重构遗漏关键约束 | LOW | v1 9 个标注全部 close 且处理结果记录在 spec-v1.md；本次 v2 重写可追溯 |
| R9 | AGENTS.md §7 强制条款过严，老 Feature 修复（F109/F110 等）实施时返工 | MED | §7 仅约束"新 Feature"与"编码启动前"，老 Feature 修复（已有测试）豁免；C-P4 已隔离 F201 |
| R10 | design.md §N 章节模板太重，小 Feature（< 5 FR）文档负担过大 | MED | 模板允许"小 Feature 简化版"——FR ≤ 5 时合并 L1~L5 表为单表，仅保留必填项 |

## 6. Open Questions

| # | 问题 | 决策时机 |
|---|------|---------|
| OQ-1 | 测试等级矩阵文档放在 `docs/test-trace-matrix.md`（F113 产物扩展）还是独立 `docs/test-level-matrix.md`？ | design 阶段 |
| OQ-2 | tauri-plugin-wdio 是 dev-only feature gate 还是直接进生产 Cargo.toml？ | design 阶段（C-P3） |
| OQ-3 | 网络/环境隔离方案选 P1 / P2 / P3 / P4 哪个？或组合？ | design 阶段 |
| OQ-4 | 网络/环境隔离是否引入容器化（推翻 C-T4）？ | design 阶段（若 P4） |
| OQ-5 | e2e/ 是否纳入 pnpm workspace（统一 lockfile）？还是保持独立 npm 包？ | design 阶段 |
| OQ-6 | tauri-plugin-wdio 引入后对 GoGuo 二进制体积 / 启动时间的量化影响？ | tasks 实施时 |
| OQ-7 | helpers 是否需要单独的单元测试套（vitest in e2e/）？还是仅靠 spec 间接验证？ | design 阶段 |
| OQ-8 | AGENTS.md §7 是否追溯强制已立项但未完成的 Feature（F109/F110/F114 等）？还是仅对未来 Feature？ | design 阶段 |
| OQ-9 | design.md §N 章节模板的"小 Feature 简化版"阈值（FR ≤ 5？≤ 10？）？ | design 阶段 |

## 7. Out of Scope（本 feature 不做）

| 项 | 推到 / 处理 |
|----|------|
| **GitHub Actions / 其它 CI 集成** | **不引入**（v1 id:09 撤；推到稳定版本后单独立项评估） |
| 6 前端页面 E2E 集中规划（dashboard / sites / rules / diagnostics / settings / wizard） | **不集中规划**（v1 id:05）；各 Feature 在自己 spec 阶段决定是否新增 e2e 覆盖 |
| F201 的 e2e spec 实际编写 | F201 自己的 design/tasks 阶段（本 feature 仅提供接入规范） |
| Windows / macOS 平台支持 | 后续 feature（CI 立项时同步评估） |
| 视觉回归测试（@wdio/visual-service） | 后续 feature |
| 视频录制（wdio-video-reporter） | 后续 feature |
| Deep link / protocol handler 测试 | 后续 feature |
| Mock IPC 注入测试（@wdio/tauri-service mock 特性） | 后续 feature |
| 缩短 waitforTimeout（评估后排除，稳定性优先） | 不做 |

## 8. 里程碑（建议）

| 阶段 | 内容 | 依赖 |
|------|------|------|
| M1 design.md | 9 个 Open Questions 决策 + L1~L5 矩阵草稿（F201 首案例）+ tauri-plugin-wdio 引入方案 + 网络隔离方案选型 + AGENTS.md §7 与 §N 模板最终稿 | spec.md v2 通过 review |
| M2 tasks.md | 拆解为可执行 task（目录重构 / 单 Feature 入口 / 3 优化 / 矩阵 / 接入规范 / AGENTS.md / 模板 / 网络隔离 / 文档） | design.md 通过 review |
| M3 基础设施规范化 | e2e/ 目录重构 + helpers 抽取 + 主仓库入口（FR-2.1 含单 Feature 入口 R4~R7） | M2 |
| M4 97s 优化实施 | 3 个优化杠杆逐项 RED-GREEN（FR-2.2，含 L1 单测） | M3 |
| M5 矩阵 + 接入规范 | 测试等级矩阵落盘（F201 首案例，结构完整）+ AGENTS.md §7 + 章节模板（FR-2.3 / FR-2.4 含 §7/§N/检查点） | M3 |
| M6 网络/环境隔离 | 选定方案实施 + 三并发场景验证（FR-2.5） | M3 |
| M7 文档同步 | testing-principles.md / PoC report / 根 README 同步（FR-2.6） | M4/M5/M6 |
| M8 finalize | flakiness 验证（10 连跑）+ F201 接入流程演练 + closeout | M4/M5/M6/M7 |

## 9. 验收证据清单（finalize 前必须产出）

- [ ] `e2e/specs/f114-baseline/{smoke,ipc}.spec.ts` 迁移完成，`e2e/test/` 删除
- [ ] `e2e/helpers/{tauri-ipc,wait,env}.ts` 存在，spec 无 inline `__TAURI_INTERNALS__`
- [ ] 仓库根 `package.json` 含 `test:e2e` / `test:all` / `test:feature` / `test:e2e:feature` 四入口
- [ ] `pnpm test:feature -- f114-baseline` 与 `pnpm test:e2e:feature -- f114-baseline` 跑通
- [ ] 本地 5 次连跑均值 ≤ 70s（时间戳证据）
- [ ] wdio 运行日志无 "Tauri plugin not available" 警告
- [ ] 测试等级矩阵文档（`docs/test-level-matrix.md` 或合并入 `docs/test-trace-matrix.md`）存在，F201 行齐全（阶段 1 结构完整）
- [ ] **`AGENTS.md` §7 "Feature 自动化测试设计强制规范" 落地**（grep 验证）
- [ ] **`docs/principles/test-design-section-template.md` 章节模板落地**
- [ ] `e2e/README.md` 接入流程文档含 Step 1~5
- [ ] F115 design.md 含完整 §N "L1~L5 自动化测试设计"章节（模板自验证）
- [ ] Feature 接入规范文档（`e2e/README.md` 章节）存在，F201 演练通过
- [ ] 网络/环境隔离方案文档 + 三并发场景验证脚本
- [ ] `cargo test --workspace && pnpm test` 全过（无回归）
- [ ] L1 单元测试覆盖 env helpers / wdio 配置 / 注册条件分支（FR-2.2.4-R3）
- [ ] flakiness 10 次连跑报告 ≤ 10%
- [ ] F114 PoC report §7.1 立即项已标注"已在 F115 实施"
- [ ] `docs/principles/testing-principles.md` 新增 L1~L5 决策原则小节
- [ ] 仓库根 `README.md` 测试入口更新为 `pnpm test:all`
