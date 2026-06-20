# Feature 115: UX E2E 自动化测试基础设施正式化 — 需求规格

- **Feature**: 115-ux-e2e-infrastructure
- **阶段**: `hf-specify`
- **状态**: 草稿 v3（基于 v2 评审 + OQ 全决策 + 多实例行为代码调研收敛 FR-2.5 范围）+ **v3 勘误（2026-06-19）**：design review id:01~06 处理结果回写 + **v3 勘误-2（2026-06-19）**：T-04 实施发现 IPC 缺口，补 FR-2.2.5 + **v3 勘误-3（2026-06-20）**：T-08 实施发现 @wdio/tauri-service v1.1.0 不支持 `skipDriverSpawn`，FR-2.2.2-R1 复用模式改走"绕开 service"路径；T-09 实施发现 design §2.3 漏列 Steps 3-6（capabilities + withGlobalTauri + npm 包 + 前端 import），补 FR-2.2.3-R2 实施细则 + **v3 勘误-4（2026-06-20）**：T-12 度量发现 T-09 后复用模式不再快于自启模式，FR-2.2.2-R3 方程翻转，重新定位"复用模式"为开发体验可选项
- **日期**: 2026-06-18（v3 发布）/ 2026-06-19（v3 勘误 + v3 勘误-2）/ 2026-06-20（v3 勘误-3 + v3 勘误-4）
- **作者**: Teddy（项目管理者 & QA guardian）
- **v3 勘误回写清单**（2026-06-19，来自 design review）：
  - id:03 → §1.1 末尾新增"术语定义"表（dev 模式 vs 生产模式）+ 衍生约束
  - id:02 → FR-2.2.2-R1 补"仅 dev 模式适用"+ dev 模式约束
  - id:05 → FR-2.4.3-R4 commit message 更新 + AGENTS.md §7 仅入口策略 + testing-principles.md 详细条款落盘
  - id:06 → FR-2.4.4-R2 补 L4/L5 UX 用例写法约束（操作序号/操作描述/期望结果列）
  - 说明：本勘误仅澄清既有 FR 的实施约束，未新增/删除 FR，不触发 v3.1 评审
- **v3 勘误-2 回写清单**（2026-06-19，来自 T-04 实施 design↔impl gap 发现）：
  - T-04 实施 `resetGoGuoState()` helper 时发现 design.md §2.1.3 引用的 `list_target_sites` IPC 命令在 GoGuo 后端不存在（`SiteRuleEngine.active_sites` 为内部私有字段，无 IPC 暴露）
  - 处理决策：选项 A（新增 IPC 命令，根本性修复）→ 新增 **FR-2.2.5** 要求后端补 `list_target_sites` 命令
  - 新增 **SC-10**：e2e helper 可枚举已添加站点，state 隔离 helper 可正常工作
  - 影响范围：design.md §2.1.3 / T-04 拆分为 T-04a（后端）+ T-04b（helper） / M3 工期 3.25d → 3.75d
  - 说明：此勘误新增 1 个 FR（属本 Feature 修复范围内，不构成新功能扩张，不触发 v3.1 评审）；本 Feature 范围因此明确包含「为支持 e2e 状态隔离所需的最小后端只读查询命令」
- **v3 勘误-3 回写清单**（2026-06-20，来自 T-08 + T-09 实施 design↔impl gap 发现）：
  - **T-08 部分**：实施时实测 `@wdio/tauri-service` v1.1.0 **不支持** `skipDriverSpawn` 选项（`grep -rn "skipDriverSpawn" node_modules/@wdio/tauri-service/` 零命中；`TauriLaunchService.onPrepare()` → `DriverPool.startDriver()` 无条件 spawn）
    - 处理决策：选项 A（双配置绕开 service）→ 复用模式从 services 数组中**移除** `@wdio/tauri-service`，改用 capabilities + 顶层 hostname/port 直连外部 tauri-driver
    - 影响范围：FR-2.2.2-R1 描述更新 / design.md §2.2.2 代码片段重写 / T-08 验收标准新增"复用模式 services 数组为空"
    - 不变更：FR-2.2.2-R2/R3/R4 要求（脚本必须存在、≥8s 下降、README 文档化）—— 这些通过新路径仍可全部满足
  - **T-09 部分**：实施时实测仅做 design §2.3 Step 1-2（Cargo.toml + lib.rs 注册）**不能消除** "Tauri plugin not available" 告警
    - 根因：plugin 完整集成需官方文档 6 步（[plugin-setup.md](https://github.com/webdriverio/desktop-mobile/blob/main/packages/tauri-service/docs/plugin-setup.md)），design §2.3 漏列 Steps 3-6
    - 处理决策：选项 α（补全 6 步）→ 新增 Step 3 capabilities + Step 4 withGlobalTauri + Step 5 npm 包 + 前端 import + Step 6 build 验证
    - 影响范围：FR-2.2.3-R2 实施细则扩展（仍是一句"在 Builder 链注册"，但 design §2.3.1 完整列出 6 步） / design.md §2.3 补 Steps 3-6
    - 风险评估：`withGlobalTauri=true` 对 GoGuo 既有 IPC（用 `@tauri-apps/api/core` wrapper）无破坏（wrapper 走 `__TAURI_INTERNALS__`，与 `__TAURI__` 叠加共存）；737 cargo tests + 220 vitest 全过（5 个 pre-existing 失败已确认非 T-09 引入）
    - 说明：此勘误未新增/删除 FR，仅修正实施路径，不触发 v3.1 评审
- **版本历史**:
  - `spec-v1.md`（v1 评审，9 项标注）
  - `spec-v2.md`（v2 评审，3 项标注；FR-2.5 仍含 P1~P4 候选与三场景 ABC）
- **上游输入**:
  - `features/114-ui-e2e-poc/poc-report.md` — PoC 可行性证据（QG1/2/3 全过）
  - `e2e/` — PoC 阶段产物（smoke + ipc 两 spec、wdio.conf.ts、独立 npm 包）
  - `docs/principles/testing-principles.md` — 三层测试方法论（宪法层）
  - `docs/principles/architectural-health-during-tdd.md` — TDD 纪律
  - `release/data/mihomo/config.yaml` — 开发态流量阻断根因
  - `features/201-first-run-baseline-confirm/spec.md` — 首个接入基础设施的下游 Feature（作为矩阵首案例）
  - `src-tauri/src/managers/mihomo_manager.rs:144-150` — mihomo adopt 机制（多实例行为调研依据）
  - `src-tauri/src/adapters/{wsl.rs:255, linux.rs:27}` — `/etc/environment` 路径硬编码（GAP-F115-1 来源）

## 1. 概述

### 1.1 目的

F114 PoC 已证明 **tauri-driver + WebDriverIO + @wdio/tauri-service** 在 WSL2/Linux 下可建立可持续的桌面 E2E 基础设施。本 feature 将 PoC **正式化为项目级基础设施**，核心交付：

1. **项目级 e2e 基础设施**：规范化 e2e/ 目录结构、helpers / fixtures 分层，可被任意后续 Feature 直接复用
2. **Feature 接入规范**：明确后续 Feature（如 F201）如何按规范添加 e2e spec，无需自行搭建
3. **97s 优化**：实施 PoC 报告 §7 标记的 3 个优化杠杆，单次跑目标 ≤ 70s（节省 ≥ 25%）
4. **测试等级矩阵**：建立 L1~L5 完整测试分工原则，以 **F201 为首个应用案例**覆盖其全部 UI/非 UI 能力的等级标注
5. **开发环境配置文档化（含多实例已知限制声明）**：将 PoC 阶段已验证的镜像绕过方案落盘为文档 + 一键脚本；显式声明多实例共存场景下的已知限制（`/etc/environment` 覆盖等），这些限制的**根治推到 F116+**（见 F110 §12 / GAP 索引文档 §9 同步记录）

**不集中规划 spec 覆盖**：本 feature 仅保留 F114 的 smoke + ipc 两 spec 作为基线（沿用），后续每个 Feature 在自己 spec 阶段决定是否新增 e2e 覆盖，不存在"F116+ 集中做 6 页面"的规划。

**不引入 CI/CD**：当前项目未发布稳定版本 + 单人开发，保留本地直推 main 的现状。CI 集成推到稳定版本之后再评估。

**术语定义**（F115 design review id:03 回写，2026-06-19）：

| 术语 | 定义 | 运行位置 | e2e 测试目标 |
|------|------|---------|------------|
| **生产模式** | goguo 面向最终用户的运行形态，通过安装程序部署 | `<install-dir>/`（典型 `~/apps/goguo/`） | ❌ 否 |
| **dev 模式** | goguo 面向开发者的运行形态，分 debug/release 两种构建 | `<项目根目录>/target/debug/goguo` 或 `<项目根目录>/target/release/goguo` | ✅ 是 |

**衍生约束**：F115 全部交付（e2e 基础设施 / 97s 优化 / 矩阵 / 接入规范 / 开发环境配置）**仅适用于 dev 模式**。生产模式不启动 tauri-driver、不跑 wdio、不依赖 tauri-plugin-wdio（生产二进制虽含 plugin 注册，但运行期不影响最终用户）。

### 1.2 范围

| 维度 | 入 | 出 |
|------|----|----|
| 项目级 e2e 基础设施 | e2e/ 目录规范化、helpers/fixtures 分层、wdio.conf 优化 | 各 Feature 自建 e2e（已被基础设施承接） |
| Feature 接入规范 | 接入文档（§2.4）+ 矩阵 + helpers 复用契约 | 强制各 Feature 接入（建议性规范） |
| 97s 优化 | 3 杠杆（cross-spec session / driver 本地常驻 / tauri-plugin-wdio 注册） | 缩短 waitforTimeout（评估后排除，稳定性优先） |
| 测试等级矩阵 | F201 全部 UI/非 UI 能力的 L1~L5 标注 | F201 之外其它 Feature 的标注（各 Feature 自行） |
| 开发环境配置文档化 | 镜像绕过方案文档化 + setup-dev-env.sh 一键脚本 + 多实例已知限制清单 | P1/P2/P3/P4 任一隔离方案实施；mihomo config 改动；`/etc/environment` 路径参数化 |
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
| **mihomo adopt 机制** | `MihomoManager::start()` 检测到 API 端口已响应时直接复用已有 mihomo 进程，不 spawn 新进程（`mihomo_manager.rs:144-150`），多 goguo 实例自然共享单 mihomo |
| **GAP-F115-N** | F115 调研发现的待后续完善问题编号，记录于 F110 design.md §12 + GAP 索引文档 §9 |

### 1.4 成功标准

| # | 标准 | 验证方式 |
|---|------|---------|
| SC-1 | 项目级 e2e 基础设施规范化：e2e/ 目录结构、helpers、fixtures、wdio.conf 全部按 §2.4 定义落地 | 目录树 + 文件存在性检查 |
| SC-2 | 单次 e2e 跑（smoke + ipc）本地 WSL2 耗时 ≤ 70s（baseline 95s，节省 ≥ 25%） | 本地 5 次连跑均值，时间戳证据 |
| SC-3 | `tauri-plugin-wdio` 在 GoGuo src-tauri 注册，wdio 运行日志无 "Tauri plugin not available" 警告 | wdio 日志 grep |
| SC-4 | 测试等级矩阵文档落盘，**以 F201 为首个案例**：覆盖 F201 spec 全部 UI 界面（L4/L5 标注）+ 非 UI 能力（L1/L2/L3 标注） | 矩阵文档存在，F201 行数 ≥ F201 spec 的 FR 总数 × 0.6 |
| SC-5 | Feature 接入规范落盘：**AGENTS.md §7 强制条款** + `docs/principles/test-design-section-template.md` 章节模板 + `e2e/README.md` 接入流程；F201 作为首个按规范接入的 Feature 验证通过 | AGENTS.md / 模板文档 / e2e README 三处落盘；F201 design.md 含完整 §N 章节 |
| SC-6 | 开发环境配置文档化：`e2e/scripts/setup-dev-env.sh` + `e2e/README.md` Step 0 + "已知限制"节落盘；脚本执行后 `cargo install tauri-driver` + `pnpm install` 在 WSL2 成功 | 脚本执行成功；README 三处内容存在 |
| SC-7 | 本地 10 次连跑 flakiness ≤ 10%（baseline PoC 0%） | 本地脚本 10 连跑报告 |
| SC-8 | tauri-plugin-wdio 引入后已有 625 个测试（455 后端 + 24 集成 + 146 前端）100% 通过 | `cargo test --workspace && pnpm test` 全过 |
| SC-9 | 单 Feature 测试入口可用：`pnpm test:feature -- f114-baseline` 与 `pnpm test:e2e:feature -- f114-baseline` 均跑通 | 命令执行成功，仅命中指定 feature 测试 |
| SC-10 | e2e 状态隔离可枚举已添加站点：`list_target_sites` IPC 命令存在，`resetGoGuoState()` helper 能正确清理（FR-2.2.5 落地，2026-06-19 v3 勘误-2 新增） | 后端命令存在 + FR 验收测试通过 + state.test.ts 三边界通过 |

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
  ├── scripts/                     # 开发环境辅助脚本
  │   └── setup-dev-env.sh         # 镜像绕过一键配置
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
- FR-2.2.2-R1: e2e/wdio.conf.ts 必须支持两种模式（**仅 dev 模式适用**，术语见 §1.1，design review id:02 回写 2026-06-19；**v3 勘误-3**：@wdio/tauri-service v1.1.0 不支持 `skipDriverSpawn`，复用模式走"绕开 service"路径）：
  - **自启模式**（默认）：`TAURI_DRIVER_REUSE` 未设或为 0，wdio 通过 @wdio/tauri-service 自动 spawn tauri-driver
  - **复用模式**（v3 勘误-3 修正）：`TAURI_DRIVER_REUSE=1`，wdio 配置 `services: []`（移除 @wdio/tauri-service），capabilities 直填 `hostname: "127.0.0.1"` + `port: TAURI_DRIVER_PORT ?? 4444`，连接外部预启的 tauri-driver
  - 验收测试: 两种模式下 `pnpm test` 均能跑通；复用模式下 wdio 启动日志显示**无** `tauri-driver ready:` 行（证明未 spawn）
  - **dev 模式约束**: tauri-driver 是 e2e 测试专用工具，生产模式（`<install-dir>/goguo`，面向最终用户）不启动 tauri-driver、不跑 wdio；多实例共存（生产 + dev）场景下 tauri-driver 仅连接 dev 版 WebDriver 端口
- FR-2.2.2-R2: 必须提供本地常驻启动脚本 `e2e/scripts/start-driver.sh`，开发者本地预启 tauri-driver 用
  - 验收测试: 脚本执行后 `ss -ltn | grep 4444` 显示端口监听
- FR-2.2.2-R3: ~~实施后复用模式下单次跑耗时下降 ≥ 8s（对比自启模式）~~ **（v3 勘误-4，2026-06-20）**：T-12 实测发现此要求在 T-09 后**不再成立** —— 复用模式 5 次均值 43.82s（stddev 14.90s），自启模式 5 次均值 28.95s（stddev 2.40s），复用反而慢 14.87s。根因假设：tauri-driver 跨多次 reuse 累积状态（前 3 次稳定 33~34s，run 4/5 退化到 52s/66s）。**重新定位**：复用模式作为 **dev 体验可选项**（避免每次 spawn driver 的固定开销 + 便于迭代调试），不强求快于自启；SC-2 核心达标由自启模式承担（28.95s ≤ 70s，余量 41s）
  - 验收测试: ~~两种模式各跑 5 次均值对比~~ 改为：自启模式 5 次均值 ≤ 70s（即 SC-2）；复用模式稳定性 stddev ≤ 10s（**当前 14.90s 不达标，挂账 F116+ 排查**）
- FR-2.2.2-R4: README 必须说明两种模式的使用场景：日常开发用复用模式（快）、首次/CI 验证用自启模式（隔离）

#### FR-2.2.3 注册 tauri-plugin-wdio

**要求**:
- FR-2.2.3-R1: `src-tauri/Cargo.toml` 必须添加 `tauri-plugin-wdio` 依赖，版本与 @wdio/tauri-service v1.1.0 声明兼容
  - 验收测试: `cargo build --release` 成功
- FR-2.2.3-R2: `src-tauri/src/lib.rs` 必须在 tauri::Builder 链中注册 `.plugin(tauri_plugin_wdio::init())`，**且配套**（v3 勘误-3）：
  - `src-tauri/capabilities/default.json` 加 `wdio:default` 权限
  - `src-tauri/tauri.conf.json` 加 `withGlobalTauri: true`
  - GoGuo 前端安装 `@wdio/tauri-plugin` 并在 `src/main.tsx` 加 `import '@wdio/tauri-plugin'`（副作用 import，注册 `window.wdioTauri`）
  - 验收测试: 编译通过，二进制能启动
- FR-2.2.3-R3: 实施后 wdio 运行日志不再出现 "Tauri plugin not available. Make sure @wdio/tauri-plugin is installed" 警告
- FR-2.2.3-R4: 必须验证 tauri-plugin-wdio 不影响 GoGuo 生产功能：已有 625 测试全过、6 个前端页面手动冒烟通过
  - 验收测试: `cargo test --workspace` 全过、`pnpm test` 全过
- FR-2.2.3-R5: 必须在 design.md 评估 dev-only feature gate 可行性，决策"是否仅 test profile 启用"或"全 profile 启用"
- FR-2.2.3-R6: 若选择全 profile 启用，必须在 ADR-0008 记录"生产 Cargo.toml 引入测试专用 plugin"的取舍

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

#### FR-2.2.5 后端 `list_target_sites` IPC 命令（v3 勘误-2 新增，2026-06-19）

> **背景**：T-04 实施 `resetGoGuoState()` 时发现 design.md §2.1.3 假设后端有 `list_target_sites` 命令，但实际 GoGuo 后端不存在。`SiteRuleEngine.active_sites` 为内部 `Vec<String>` 字段，已持久化到 `<data_dir>/config/active-sites.json`，但无 IPC 暴露。本 FR 补齐这个**只读查询命令**，是 e2e 状态隔离的前置依赖。

**要求**:
- FR-2.2.5-R1: **复用既有** `src-tauri/src/engines/site_rule_engine.rs:521` 的 `pub const fn active_sites(&self) -> &Vec<String>` 访问器（无需新增 engine 层方法）
  - 验收测试: 实施时确认该访问器仍存在且为 `pub`
  - 可观测结果: Tauri 命令能通过该访问器拿到当前 active_sites 快照
- FR-2.2.5-R2: `src-tauri/src/commands/site_rules.rs` 必须新增 `#[tauri::command] pub fn list_target_sites(state: tauri::State<'_, SiteRulesState>) -> Vec<String>`，内部调用 `engine.active_sites().clone()`
  - 验收测试: FR 验收测试（`src-tauri/tests/fr_acceptance/`）覆盖：空列表、单站点、多站点、增删后查询四个 case
  - 可观测结果: IPC 调用返回的 site id 列表与 `add_target_site` / `remove_target_site` 操作序列一致
- FR-2.2.5-R3: `src-tauri/src/lib.rs` 的 `tauri::Builder::invoke_handler!` 必须注册 `list_target_sites`
  - 验收测试: 前端 `invoke('list_target_sites')` 能拿到非 error 响应
- FR-2.2.5-R4: `src/lib/tauri-ipc.ts` 必须新增 `export async function listTargetSites(): Promise<string[]>` wrapper
  - 验收测试: wrapper 存在，e2e helpers/state.ts 通过该 wrapper 调用
- FR-2.2.5-R5: 该命令必须为**只读**（不修改 `active_sites`，不触发 mihomo reload，不写审计日志），保证 e2e `beforeEach` 高频调用安全
  - 验收测试: 命令实现无 `MutexGuard<Engine>` 的可变操作；FR 测试断言连续调用 2 次返回值相同

### 2.3 测试等级矩阵（L1~L5）

#### FR-2.3.1 矩阵文档建立

**要求**:
- FR-2.3.1-R1: 必须建立测试等级矩阵文档（位置由 OQ-1 决策为独立 `docs/test-level-matrix.md`），与 F113 的 `docs/test-trace-matrix.md` **并存且双向链接**
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
- FR-2.4.3-R1: 项目根 `AGENTS.md` 必须新增章节 **"§7. Feature 自动化测试设计强制规范"**（编号可由现有结构决定，作为 §6 后续），声明以下硬约束（适用于所有未来 Feature，F115 自身豁免以避免循环依赖；显式豁免清单：F109/F110/F114/F115/F101~F106 已立项但未启动项）：
  - **强制条款 1**: 每个新立项 Feature 或新启动的 fix Feature 在 `hf-design` 阶段必须输出一节"§X L1~L5 自动化测试设计"（章节位置由 §2.4.4 模板定义），作为 design.md 的必填章节
  - **强制条款 2**: 该章节未通过 review **不得进入** `hf-tasks` 阶段（即编码启动前必须完成测试设计）
  - **强制条款 3**: 章节必须同时覆盖 **UX 能力（L4/L5）**与**非 UI 能力（L1/L2/L3）**，对每条 FR 给出测试用例 + 数据 + 脚本入口
  - **强制条款 4**: `hf-finalize` 阶段必须验证该章节中所有声明的测试均已实现且通过，否则不通过完成门
- FR-2.4.3-R2: `AGENTS.md` §4 "Coding / Testing / Architecture 标准"必须交叉引用 §7 与 `docs/test-level-matrix.md`、`docs/principles/testing-principles.md`
- FR-2.4.3-R3: 仓库根 `README.md` "Active feature 指针来源"附近必须新增一句"所有 Feature design 必须含 L1~L5 自动化测试设计章节（详见 AGENTS.md §7）"
- FR-2.4.3-R4: AGENTS.md 改动必须作为 F115 tasks 阶段交付物（spec 阶段只定义"要改什么"），实际 commit message 遵循 `docs(agents+testing-principles): enforce L1~L5 test design section per feature`（**id:05 回写**：两文档同 commit）
  - **AGENTS.md §7 仅入口策略**（id:05 回写，2026-06-19）：AGENTS.md §7 仅含 **4 条强制条款摘要（每条 1 行）+ 引用关系**；**详细条款**（HF 全流程检查点表、§N 完整模板引用、豁免清单详细说明、L1~L5 等级决策原则）落盘到 `docs/principles/testing-principles.md` 新增小节"L1~L5 自动化测试设计强制规范"
  - 验收测试: `grep -n "Feature 自动化测试设计强制规范" AGENTS.md` 返回非空；`grep -n "L1~L5 自动化测试设计强制规范" docs/principles/testing-principles.md` 返回非空
  - 可观测结果: AGENTS.md §7 含 4 条摘要 + 引用（不含详细检查点表）；testing-principles.md 含完整检查点表 + 豁免清单 + 等级原则
- FR-2.4.3-R5: F115 自身实施完成后必须 commit 保证 workspace 干净（不混入未提交的 F115 改动），便于后续 Feature 在干净基线上接入 §7 规范
  - 验收测试: F115 finalize 后 `git status` 显示 working tree clean（或仅含 closeout 文档）

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
  - **"小 Feature 简化版"阈值**（OQ-9 决策）：当 Feature FR 总数 ≤ 5 时，可使用单表简化结构（必填：FR ID / 能力 / L1~L5 / 关联 FR），N.3/N.4/N.5 作为可选项保留（Feature 有特殊数据/脚本/顺序需求时附加）；模板文档头部须明示阈值与"必填 vs 可选"清单
  - **L4/L5 UX 用例写法约束**（id:06 回写，2026-06-19）：模板 N.2.4 / N.2.5 表格列结构必须为 `| spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR |`（通用 UX 测试用例写法），不再使用"describe/it + 渲染场景"抽象描述；操作序号、操作描述、期望结果为必填列
  - 验收测试: 文件存在；F115 design.md 含完整 §N 章节；模板 L4/L5 子节含「操作序号 / 操作描述 / 期望结果」列
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

### 2.5 开发环境配置文档化

> **范围收敛说明**（v2→v3 重大修订）：基于 `mihomo_manager.rs:144-150` 的 adopt 机制代码事实调研，多 goguo 实例并存时 mihomo 进程层**不冲突**（第二个实例 adopt 已有 mihomo）。真正的多实例问题（`/etc/environment` 互相覆盖）属于 **goguo 后端硬编码**（`wsl.rs:255` / `linux.rs:27`），不在 F115 范围内修复，已登记为 GAP-F115-1 推到 F116+。本节仅处理"e2e 执行期间 cargo/pnpm 流量被 mihomo 阻断"的开发态配置问题。

#### FR-2.5.1 问题边界

**要求**:
- FR-2.5.1-R1: 必须在 design.md 明确问题边界——e2e 执行期间，GoGuo release/debug 二进制启动后会启动 mihomo 子进程（基于 `release/data/mihomo/config.yaml`），其 `site-crates` / `site-npmjs` ruleset 存在但无 DIRECT 规则匹配，阻断 cargo/pnpm 流量（PoC 已实测 `SSL_ERROR_SYSCALL`）。F115 仅文档化 PoC 阶段已验证的镜像绕过方案，**不改 mihomo config**（C-I5）、**不修复根因**（推到 F116+，见 GAP-F115-2）
- FR-2.5.1-R2: 必须明确**多实例行为说明**（基于代码事实）：
  - mihomo 进程层：第二个 goguo 实例通过 adopt 机制复用已有 mihomo，无端口冲突（`mihomo_manager.rs:144-150`）
  - `/etc/environment` 层：硬编码路径，多实例互相覆盖（`wsl.rs:255` / `linux.rs:27`），**非 F115 解决**（GAP-F115-1）
  - cargo/pnpm 流量层：与多实例无关，单实例下也存在（GAP-F115-2）
- FR-2.5.1-R3: 必须显式声明 **Out of Scope** 的多实例场景（详见 §7）：
  - 场景 1: 仅 `~/apps/goguo` 运行（用户日常使用，非 e2e）
  - 场景 2: `~/apps/goguo` + `target/debug/goguo`（开发者日常 + 调试）
  - 场景 3: `~/apps/goguo` + `target/release/goguo`（开发者日常 + e2e）
  - **`/etc/environment` 路径参数化**（goguo 后端 Feature，GAP-F115-1）
  - **mihomo config dev/prod 拆分**（GAP-F115-3）

#### FR-2.5.2 配置文档化方案

**要求**:
- FR-2.5.2-R1: 必须在 `e2e/README.md` 新增 **"Step 0: 开发环境首次配置"** 章节，文档化镜像绕过方案：
  - `~/.cargo/config.toml` 加 `rsproxy-sparse` 镜像源（处理 cargo 流量阻断）
  - `e2e/.npmrc` 已含 npmmirror.com（C-I4 隔离策略，处理 pnpm 流量阻断）
  - 校验步骤：`cargo install tauri-driver --dry-run` + `pnpm install --dry-run` 不报 SSL 错误
- FR-2.5.2-R2: 必须提供一键配置脚本 `e2e/scripts/setup-dev-env.sh`，自动化 Step 0：
  - 检测 `~/.cargo/config.toml` 是否已含 rsproxy-sparse；若无则追加
  - 检测 `e2e/.npmrc` 是否存在；若无则报错（C-I4 强制项）
  - 幂等：重复执行不产生重复配置
  - 验收测试: 在干净环境执行脚本后，cargo/pnpm 命令可正常工作
  - 可观测结果: 脚本输出"配置完成"且 `cargo install tauri-driver` 不再 SSL 失败
- FR-2.5.2-R3: 必须在 `e2e/README.md` 新增 **"已知限制"** 章节，显式声明：
  - 多实例 `/etc/environment` 覆盖问题（指向 F110 §12 GAP-F115-1）
  - mihomo config 流量阻断根因未修复（指向 F110 §12 GAP-F115-2）
  - 当前缓解仅覆盖开发态镜像绕过，**不影响**生产用户行为（C-I5 保持）
- FR-2.5.2-R4: 必须在 design.md 评估是否引入 P3（HOME 隔离）作为可选增强；基于代码事实（adopt 机制已处理 mihomo 共存），**默认不引入** P3，仅在 GAP-F115-1 修复后评估

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
- NFR-3.4.2: 开发环境配置文档化方案必须**不影响 GoGuo release 的生产用户行为**——不动 `release/data/mihomo/config.yaml`（C-I5）；多实例 `/etc/environment` 覆盖问题作为 GAP 推到 F116+，F115 仅文档声明

## 4. 约束

### 4.1 不变量（C-Invariants）

- C-I1: 不修改 F114 已验证的 PoC 关键配置：`browserName: "tauri"`、`tauri:options.application`、`GDK_BACKEND=x11`
- C-I2: 不引入新的 browserName（chrome/firefox 等）——仅 Tauri webview
- C-I3: 不删除 F114 的 smoke.spec.ts / ipc.spec.ts 测试覆盖（可迁移路径与重构结构，但测试用例与断言不变）
- C-I4: 不改变 e2e/.npmrc 镜像隔离策略（避免污染主工程）
- C-I5: **不修改 `release/data/mihomo/config.yaml` 现有生产规则**（含不新增 `site-dev-tools` ruleset；如需开发态放行，推到 F116+ 处理 mihomo config 设计层）

### 4.2 技术约束

- C-T1: tauri-plugin-wdio 版本必须与 @wdio/tauri-service v1.1.0 声明兼容的版本
- C-T2: 不引入 Selenium Grid 或其它外部 driver 编排服务
- C-T3: 不引入 Playwright、Cypress 或其它 E2E 框架（与 F114 选型一致）
- C-T4: 不引入 Docker/Podman 容器化（OQ-3/OQ-4 决策：仅文档化镜像绕过方案，多实例问题推 F116+）
- C-T5: 不引入 GitHub Actions 或其它 CI 平台（撤 CI，v1 id:09）

### 4.3 流程约束

- C-P1: F115 spec 通过 review 后才能进入 design 阶段（interactive 模式）
- C-P2: 97s 优化每个杠杆必须先在本地验证效果再合入 main
- C-P3: tauri-plugin-wdio 引入必须先在 design.md 评估 dev-only feature gate 可行性（OQ-2 决策：进生产 Cargo.toml + ADR-0008）
- C-P4: F201 接入规范验证（FR-2.4.1-R2）必须在 F201 design 阶段而非 F115 内完成（F115 仅提供规范与基础设施，不替 F201 写 spec）
- C-P5: F115 自身实施完成后必须 commit 保证 workspace 干净（FR-2.4.3-R5）

## 5. 风险登记

| # | 风险 | 等级 | 缓解 |
|---|------|------|------|
| R1 | tauri-plugin-wdio 引入后破坏 GoGuo 现有功能（插件副作用） | HIGH | FR-2.2.3-R4 全套回归；OQ-6 度量阈值（体积 ≤ 2MB / 启动 ≤ 50ms），超阈则回退 dev-only feature gate（OQ-2 备选） |
| R2 | `restart: false` 导致 spec 间状态污染 | MED | helpers 提供 `resetGoGuoState()`；spec `beforeEach` 重置；NFR-3.2.3 验证 |
| R3 | driver 复用模式下端口占用或僵尸进程 | MED | 复用脚本提供 stop-driver.sh；超时保护 |
| R4 | L1~L5 矩阵定义过于严格，后续 Feature 接入返工 | MED | 矩阵作为"指南"而非"硬约束"，design 阶段允许调整；FR-2.3.1-R4 决策原则可演进；FR-2.4.4-R2 小 Feature 简化版兜底 |
| R6 | mihomo config 阻断开发态 cargo/pnpm 流量持续影响开发（同 F114） | HIGH | FR-2.5 文档化镜像绕过 + setup-dev-env.sh 一键脚本；根因修复推到 F116+（GAP-F115-2） |
| R7 | F201 作为接入规范首个案例验证失败，规范需大改 | LOW | F201 验证（FR-2.4.1-R2）作为 F115 finalize 前置条件；早发现早调整 |
| R8 | spec-v1 → spec-v2 → spec-v3 多版本迭代遗漏关键约束 | LOW | v1/v2 评审标注全部 close 且处理结果记录在归档版本；v3 重写仅收敛 FR-2.5 范围，其它章节保持 v2 决策 |
| R9 | AGENTS.md §7 强制条款过严，老 Feature 修复实施时返工 | MED | §7 显式豁免清单：F109/F110/F114/F115/F101~F106；仅约束 F115 之后新立项或新启动 Feature（OQ-8 决策） |
| R10 | design.md §N 章节模板太重，小 Feature 文档负担过大 | MED | FR-2.4.4-R2 小 Feature 简化版阈值 FR ≤ 5（OQ-9 决策），N.3/N.4/N.5 作为可选项保留 |
| R11 | 多实例 `/etc/environment` 互相覆盖影响 ProxyGuard 准确性（`wsl.rs:255` / `linux.rs:27` 硬编码） | HIGH | **不在 F115 解决**；GAP-F115-1 已登记到 F110 §12 + GAP 索引文档 §9，建议 F116+ 立项修复 goguo 后端 |

> v2 风险表中的 R5（网络隔离方案选型返工）已删除——v3 不引入 P1~P4 任一方案，无选型返工风险。

## 6. Open Questions 决策摘要

> 所有 OQ 已在 spec 阶段完成决策，design.md 起草时直接消费。决策细节见对话记录与归档版本 spec-v2.md。

| # | 问题 | 决策 | 决策依据 / 风险控制 |
|---|------|------|--------------------|
| OQ-1 | 测试等级矩阵文档位置 | **独立 `docs/test-level-matrix.md`** | 与 F113 trace-matrix 维度不同（level 是能力→L1~L5 分工，trace 是 FR→测试函数 1:1），双向链接兜底 |
| OQ-2 | tauri-plugin-wdio 是 dev-only gate 还是生产 Cargo.toml | **生产 Cargo.toml + ADR-0008** | Tauri 官方默认路径；超 OQ-6 阈值则回退 dev-only gate |
| OQ-3 | 网络/环境隔离方案 P1/P2/P3/P4 | **不引入任一方案**；仅文档化镜像绕过 + setup 脚本 + 已知限制声明 | 代码事实调研：多实例 mihomo 通过 adopt 自洽；`/etc/environment` 覆盖是 goguo 后端硬编码问题（GAP-F115-1）；cargo/pnpm 阻断与多实例无关（GAP-F115-2） |
| OQ-4 | 是否引入容器化 | **不引入**，C-T4 保持 | OQ-3 已无 P4 需求 |
| OQ-5 | e2e/ 是否纳入 pnpm workspace | **保持独立 npm 包** | C-I4 镜像隔离策略；F114 PoC 已验证可行 |
| OQ-6 | tauri-plugin-wdio 体积/启动时间量化影响 | **design 定阈值 + tasks 实测** | 阈值：体积 ≤ 2MB / 启动 ≤ 50ms / RSS ≤ 5MB；超阈回退 dev-only gate |
| OQ-7 | helpers 是否需单独单元测试套 | **引入 vitest in e2e/** | FR-2.2.4-R3 强制 L1 单测；e2e/helpers/__tests__/ 目录 + `test:unit` script |
| OQ-8 | AGENTS.md §7 是否追溯老 Feature | **仅约束未来 Feature**，显式豁免 F109/F110/F114/F115/F101~F106 | 老 Feature 已有测试基础，强制补 §N 属返工；F201 为首个强制案例 |
| OQ-9 | §N 模板"小 Feature 简化版"阈值 | **FR ≤ 5 启用单表简化版**；N.3/N.4/N.5 作为可选项保留 | 5 是常见小 Feature 体量；超 5 条 FR 单表横向滚动严重 |

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
| **场景 1：仅 `~/apps/goguo` 运行**（用户日常使用，非 e2e 场景） | 不在 F115 范围 |
| **场景 2：`~/apps/goguo` + `target/debug/goguo` 同时运行**（开发者日常 + 调试） | 不在 F115 范围（多实例问题见下） |
| **场景 3：`~/apps/goguo` + `target/release/goguo` 同时运行**（开发者日常 + e2e） | 不在 F115 范围（多实例问题见下） |
| **多实例 `/etc/environment` 互相覆盖** | GAP-F115-1，推到 F116+（goguo 后端硬编码 `wsl.rs:255` / `linux.rs:27`） |
| **mihomo config dev/prod 拆分** | GAP-F115-3，推到 F116+（与 GAP-F115-2 合并处理） |
| **mihomo config 阻断 cargo/pnpm 流量根因修复** | GAP-F115-2，推到 F116+（mihomo config 设计层；F115 仅文档化镜像绕过） |

## 8. 里程碑（建议）

| 阶段 | 内容 | 依赖 |
|------|------|------|
| M1 design.md | OQ 已全决策（§6）+ L1~L5 矩阵草稿（F201 首案例）+ tauri-plugin-wdio 引入方案 + 开发环境配置文档化方案 + AGENTS.md §7 与 §N 模板最终稿 | spec.md v3 通过 review |
| M2 tasks.md | 拆解为可执行 task（目录重构 / 单 Feature 入口 / 3 优化 / 矩阵 / 接入规范 / AGENTS.md / 模板 / 配置文档化 / 文档同步） | design.md 通过 review |
| M3 基础设施规范化 | e2e/ 目录重构 + helpers 抽取 + 主仓库入口（FR-2.1 含单 Feature 入口 R4~R7） | M2 |
| M4 97s 优化实施 | 3 个优化杠杆逐项 RED-GREEN（FR-2.2，含 L1 单测） | M3 |
| M5 矩阵 + 接入规范 | 测试等级矩阵落盘（F201 首案例，结构完整）+ AGENTS.md §7 + 章节模板（FR-2.3 / FR-2.4 含 §7/§N/检查点） | M3 |
| M6 开发环境配置文档化 | setup-dev-env.sh + README Step 0 + 已知限制节（FR-2.5） | M3 |
| M7 文档同步 | testing-principles.md / PoC report / 根 README 同步（FR-2.6） | M4/M5/M6 |
| M8 finalize | flakiness 验证（10 连跑）+ F201 接入流程演练 + closeout + workspace clean commit | M4/M5/M6/M7 |

## 9. 验收证据清单（finalize 前必须产出）

- [ ] `e2e/specs/f114-baseline/{smoke,ipc}.spec.ts` 迁移完成，`e2e/test/` 删除
- [ ] `e2e/helpers/{tauri-ipc,wait,env}.ts` 存在，spec 无 inline `__TAURI_INTERNALS__`
- [ ] 仓库根 `package.json` 含 `test:e2e` / `test:all` / `test:feature` / `test:e2e:feature` 四入口
- [ ] `pnpm test:feature -- f114-baseline` 与 `pnpm test:e2e:feature -- f114-baseline` 跑通
- [ ] 本地 5 次连跑均值 ≤ 70s（时间戳证据）
- [ ] wdio 运行日志无 "Tauri plugin not available" 警告
- [ ] 测试等级矩阵文档 `docs/test-level-matrix.md` 存在，F201 行齐全（阶段 1 结构完整）
- [ ] **`AGENTS.md` §7 "Feature 自动化测试设计强制规范" 落地**（含豁免清单，grep 验证）
- [ ] **`docs/principles/test-design-section-template.md` 章节模板落地**（含小 Feature 简化版阈值说明）
- [ ] `e2e/README.md` 含 Step 0（首次配置）+ 接入流程（Step 1~5）+ 已知限制节
- [ ] **`e2e/scripts/setup-dev-env.sh` 一键配置脚本落地**（幂等，执行后 cargo/pnpm 可工作）
- [ ] F115 design.md 含完整 §N "L1~L5 自动化测试设计"章节（模板自验证）
- [ ] Feature 接入规范文档（`e2e/README.md` 章节）存在，F201 演练通过
- [ ] **ADR-0008 落盘**（记录"生产 Cargo.toml 引入测试专用 plugin tauri-plugin-wdio"取舍）
- [ ] `cargo test --workspace && pnpm test` 全过（无回归）
- [ ] L1 单元测试覆盖 env helpers / wdio 配置 / 注册条件分支（FR-2.2.4-R3）
- [ ] flakiness 10 次连跑报告 ≤ 10%
- [ ] F114 PoC report §7.1 立即项已标注"已在 F115 实施"
- [ ] `docs/principles/testing-principles.md` 新增 L1~L5 决策原则小节
- [ ] 仓库根 `README.md` 测试入口更新为 `pnpm test:all`
- [ ] **F110 design.md §12 + GAP 索引文档 §9 已同步记录 GAP-F115-1/2/3**（多实例问题移交）
- [ ] **F115 实施完成后 workspace clean**（`git status` 显示 clean，FR-2.4.3-R5 / C-P5）
