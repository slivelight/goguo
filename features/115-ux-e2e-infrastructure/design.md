# Feature 115: UX E2E 自动化测试基础设施正式化 — 设计文档

- **Feature**: 115-ux-e2e-infrastructure
- **阶段**: `hf-design`（M1）
- **状态**: 草稿 v1（基于 spec.md v3 起草，待 review）
- **日期**: 2026-06-18
- **作者**: Teddy（项目管理者 & QA guardian）
- **Authority Source**: [spec.md](./spec.md) v3（已通过 review）
- **上游依赖**: F114 PoC 报告、F113 三层测试方法论、F110 §12（GAP-F115-1/2/3）
- **下游应用**: F201（接入规范首个案例）、F110/F109（豁免）、F116+（GAP 修复）

---

## 1. 设计概述

### 1.1 设计目标（映射 spec §1.1）

将 F114 PoC 验证的三件套（tauri-driver + WebDriverIO + @wdio/tauri-service）正式化为**项目级基础设施**，覆盖五个交付维度：

| 维度 | spec FR | design 章节 |
|------|---------|------------|
| 项目级 e2e 基础设施 | FR-2.1 | §3 |
| 97s 优化（3 杠杆） | FR-2.2 | §2 |
| 测试等级矩阵（F201 首案例） | FR-2.3 | §4 |
| Feature 接入规范（AGENTS §7 + §N 模板） | FR-2.4 | §5 |
| 开发环境配置文档化 | FR-2.5 | §6 |

### 1.2 设计原则

1. **最小侵入生产代码**：仅 `src-tauri/Cargo.toml` 加 `tauri-plugin-wdio` + `src-tauri/src/lib.rs` 注册一行 `.plugin(...)`。其它生产文件零改动。
2. **代码事实优先**：所有"多实例行为"判断基于 `mihomo_manager.rs:144-150` 与 `wsl.rs:255` 实测，不基于推测（§7）。
3. **文档化为先，工具化为后**：先落盘接入规范与章节模板（§5），再在 F201 验证；lint 工具属可选增强（§5.3）。
4. **GAP 显式移交**：不试图在 F115 解决所有问题，多实例 `/etc/environment` 等问题已登记到 F110 §12 + GAP 索引 §9 推到 F116+。

### 1.3 关键决策摘要（来自 spec §6 OQ）

| OQ | 决策 | design 落盘章节 |
|----|------|----------------|
| OQ-1 | 独立 `docs/test-level-matrix.md` | §4.1 |
| OQ-2 | 生产 Cargo.toml + **ADR-0008** | §2.3 + §10 |
| OQ-3 | 不引入 P1~P4，仅文档化 | §6 + §7 |
| OQ-4 | 不引入容器化 | §7.1 |
| OQ-5 | 保持独立 npm 包 | §3.1 |
| OQ-6 | design 定阈值 + tasks 实测 | §2.4 |
| OQ-7 | 引入 vitest in e2e/ | §3.2 + §N.2.1 |
| OQ-8 | 仅约束 F115 后新 Feature | §5.1 |
| OQ-9 | FR ≤ 5 启用简化版 | §5.2 |

### 1.4 术语定义（? id:03 处理结果，2026-06-19）

> 本节为 F115 全文引用的术语锚点。spec.md 同步回写（§1.1 末尾或新增小节）。

| 术语 | 定义 | 运行位置 | 是否 e2e 测试目标 |
|------|------|---------|-----------------|
| **生产模式** | goguo 面向最终用户的运行形态，通过安装程序部署 | `<install-dir>/`（典型 `~/apps/goguo/`） | ❌ 否（最终用户日常使用，不跑 e2e） |
| **dev 模式** | goguo 面向开发者的运行形态，分 debug/release 两种构建 | `<项目根目录>/target/debug/goguo` 或 `<项目根目录>/target/release/goguo` | ✅ 是（e2e 测试目标） |

**衍生约束**：
- F115 e2e 基础设施的所有设计（杠杆 1~3、helpers、入口脚本、矩阵）**仅适用于 dev 模式**
- 生产模式不启动 tauri-driver、不跑 wdio、不依赖 tauri-plugin-wdio（生产二进制虽含 plugin 注册，但运行期不影响最终用户）
- 多实例共存（生产 + dev）行为由 mihomo adopt 机制自洽（详见 §7.2），不在 F115 范围调整

---

## 2. 97s 优化设计（spec FR-2.2）

### 2.1 杠杆 1：cross-spec session 复用（`restart: false`）

#### 2.1.1 现状与目标

> **场景澄清（? id:01 处理结果，2026-06-19）**：
> 杠杆 1 的收益场景是 **同一 wdio run 内多个 spec 共享 GoGuo session**，主要发生在：
> - `pnpm test:feature -- f114-baseline`：f114-baseline 下 5 个 spec（smoke + ipc + session-reuse + driver-reuse + plugin-registered）
> - `pnpm test:all` / `pnpm test:e2e`：跨 Feature 累积（F114 + F201 + ...）
>
> 用户提到的"单 spec 单独测试"（`wdio run --spec <path>`）是**调试边缘场景**，杠杆 1 无收益但不亏损（restart:false 默认行为已支持，单 spec 场景 baseline ≈ 40s）。
>
> 用户提到的"安装版 + dev 版多实例"是**独立维度**（详见 §1.4 术语定义 + §7.2 多实例行为），由 mihomo adopt 机制自洽，**与杠杆 1 不冲突**——杠杆 1 是"同一 wdio run 内 spec 间 spawn 优化"，多实例是"生产 + dev 共存"。
>
> **结论：杠杆 1 保留纳入 spec 范围**，主要收益场景为多 spec 测试入口。

| 指标 | F114 PoC baseline | F115 目标 |
|------|-------------------|----------|
| smoke + ipc 总耗时（多 spec 场景） | 95s（wdio 默认 `restart: true`，每个 spec 各 spawn GoGuo，14s × 2 = 28s spawn 开销） | ≤ 80s（session 共享，GoGuo 只 spawn 一次） |
| 单 spec 调试场景 | ≈ 40s（无 cross-spec 开销） | ≈ 40s（无变化，restart:false 不亏损） |
<!-- ? id:01;status:close;date:2026-06-19T13:30 依据之前讨论多实例运行时的用户场景，不存在双 spec 各 spawn GoGuo这种场景，也就是单个spec单独测试，但会存在安装goguo + dev goguo多实例存在场景，请确认这个优化是否还要继续纳入spec范围？；任务处理结果：保留纳入 spec 范围。主要收益场景为多 spec 测试入口（test:feature / test:all）；单 spec 调试为边缘场景无收益但不亏损；多实例场景由 mihomo adopt 自洽与杠杆 1 独立不冲突。§2.1.1 已补场景澄清段 + 修订表格描述（区分多 spec vs 单 spec），术语锚点见 §1.4 -->

#### 2.1.2 实现方案

`e2e/wdio.conf.ts` 关键改动：

```typescript
export const config: WebdriverIO.Config = {
  // ... 其它保持不变
  capabilities: [
    {
      maxInstances: 1,
      browserName: "tauri",
      "tauri:options": { application: GOGUO_BIN },
      // F115: spec FR-2.2.1-R1，多 spec 共享同一 WebDriver session
      "wdio:restartStrategy": "none",  // 等价于 restart: false（wdio v9 配置项）
    },
  ],
};
```

> **注**：wdio v9 中 `restart` 字段已迁移到 capability 层的 `wdio:restartStrategy`（v8 的根级 `restart: false` 仍兼容）。design 实施时以 wdio v9 文档为准。

#### 2.1.3 状态隔离设计（spec FR-2.2.1-R3）

每个 spec `beforeEach` 调用 `resetGoGuoState()`（位于 `e2e/helpers/state.ts`，新增）：

```typescript
// e2e/helpers/state.ts
export async function resetGoGuoState(): Promise<void> {
  // 清理 SiteRulesState（通过 IPC remove_target_site 逐个清理）
  const sites = await invokeTauri<string[]>("list_target_sites");
  for (const siteId of sites) {
    await invokeTauri("remove_target_site", { siteId });
  }
  // 重置 baseline 状态（如需要）
  // 注：goguo 后端暂无 reset_state 命令；通过 IPC 组合实现
}
```

> **✅ IPC 缺口已对齐（2026-06-19，spec v3 勘误-2）**：本节代码块引用的 `list_target_sites` IPC 命令在 T-04 实施时发现后端缺失，按 spec FR-2.2.5 处理决策 A 补齐：
> - `src-tauri/src/engines/site_rule_engine.rs:521` 既有 `pub const fn active_sites(&self) -> &Vec<String>` 访问器（无需新增 engine 层方法）
> - `src-tauri/src/commands/site_rules.rs` 新增 `#[tauri::command] pub fn list_target_sites(state) -> Vec<String>`（2 行薄壳：lock + active_sites().clone()）
> - `src-tauri/src/lib.rs` invoke_handler! 注册
> - `src/lib/tauri-ipc.ts` 新增 `listTargetSites()` wrapper
> - FR 验收测试：`src-tauri/tests/fr_acceptance/f003_site_rules.rs` 4 个 case（空 / 单 / 多 / 增删后+只读）

**测试设计**：状态隔离逻辑本身需 L1 单测（spec FR-2.2.4-R3），覆盖空状态、单站点、多站点三种边界（详见 §N.2.1）。

### 2.2 杠杆 2：tauri-driver 本地常驻（仅 dev 模式，? id:02 处理结果）

> **模式适用范围（? id:02 处理结果，2026-06-19）**：
> tauri-driver 是 e2e 测试专用工具，**仅在 dev 模式启动**（术语见 §1.4）。
> - ✅ dev 模式（`<项目根目录>/target/debug|release/goguo`）：tauri-driver 常驻或自启，跑 e2e
> - ❌ 生产模式（`<install-dir>/goguo`，如 `~/apps/goguo/`）：不启动 tauri-driver、不跑 e2e（最终用户日常使用）
>
> 多实例共存场景（生产 + dev 同时运行）下，tauri-driver 仅连接 dev 版 GoGuo 的 WebDriver 端口；生产版不受影响。

#### 2.2.1 双模式设计（spec FR-2.2.2-R1）

| 模式 | 触发 env | 行为 | 场景（仅 dev 模式） |
|------|---------|------|------|
| 自启（默认） | `TAURI_DRIVER_REUSE` 未设或 `0` | @wdio/tauri-service spawn tauri-driver | dev 模式首次配置、CI（未来）、隔离验证 |
| 复用 | `TAURI_DRIVER_REUSE=1` | wdio 跳过 spawn，连接 `TAURI_DRIVER_PORT`（默认 4444） | dev 模式日常开发（多次跑 e2e） |
<!-- ? id:02;status:close;date:2026-06-19T13:30 依据之前讨论多实例运行时的用户场景，明确tauri-driver 本地常驻是dev模式（包括debug和release），还是生产运行模式，建议仅限于dev模式；任务处理结果：采纳用户建议（仅 dev 模式）。§2.2 章节标题已加「（仅 dev 模式）」限定，§2.2.1 表格「场景」列已补「dev 模式」前缀，新增「模式适用范围」段引用 §1.4 术语锚点。spec.md FR-2.2.2-R1 待回写（补「仅 dev 模式」约束）。 -->
<!-- ? id:03;status:close;date:2026-06-19T13:30 明确dev模式和生产模式两个术语，生产模式是goguo运行在安装目录<install-dir>下，不需要自动化测试，dev模式是goguo运行在<项目根目录>/target/debug或者是<项目根目录>/target/release目录下；任务处理结果：采纳用户定义，已在 design.md §1.4 新增"术语定义"小节，明确生产模式（install-dir）vs dev 模式（target/debug|release），并声明 F115 e2e 基础设施仅适用 dev 模式。spec.md 待回写（§1.1 或新增小节） -->

#### 2.2.2 wdio.conf.ts 条件逻辑（v3 勘误-3 重写，2026-06-20）

> **原 design（v3 草稿）**：复用模式给 service 加 `skipDriverSpawn: true` 选项。
> **勘误原因**：`@wdio/tauri-service` v1.1.0 实测**不支持** `skipDriverSpawn`（grep 零命中，`TauriLaunchService.onPrepare()` 无条件 spawn）。
> **新路径**：复用模式**完全移除** service，capabilities + 顶层 hostname/port 直连外部 tauri-driver。
> 本 Feature helpers 全用 `browser.execute()` 调 `window.__TAURI_INTERNALS__.invoke()`，不依赖 service 专属 API（`browser.tauri.execute()` / mock store），绕开无功能损失。

```typescript
// e2e/wdio.conf.ts
import { shouldReuseDriver, getTauriDriverPort } from "./helpers/env";

const reuseDriver = shouldReuseDriver();
const driverPort = getTauriDriverPort();

export const config: WebdriverIO.Config = {
  // 复用模式：不引入 service，直连外部 tauri-driver
  ...(reuseDriver
    ? { services: [], hostname: "127.0.0.1", port: driverPort }
    : {
        services: [
          ["@wdio/tauri-service", { driverProvider: "external" }],
        ],
      }),
  capabilities: reuseDriver
    ? [
        {
          // 不设 browserName：service 在自启模式才注入；tauri-driver 接受裸 caps
          "tauri:options": { application: GOGUO_BIN },
        } as WebdriverIO.Capabilities,
      ]
    : [
        {
          maxInstances: 1,
          browserName: "tauri",
          "tauri:options": { application: GOGUO_BIN },
        } as WebdriverIO.Capabilities,
      ],
  // ...
};
```

> **driverProvider 注意**：v1.x 用 `"official"`，v2 已改为 `"external"`（v1 用 `"official"` 会打 deprecation 警告）。**勘误-3 决定：直接用 `"external"`**（向前兼容 v2，v1.x 同样接受该值——见 `normaliseDriverProvider()` in tauri-service/dist/cjs/index.js:1844）。

#### 2.2.3 启停脚本（spec FR-2.2.2-R2）

```bash
# e2e/scripts/start-driver.sh
#!/usr/bin/env bash
set -euo pipefail
PORT="${TAURI_DRIVER_PORT:-4444}"
if ss -ltn | grep -q ":${PORT}"; then
  echo "[start-driver] tauri-driver already listening on ${PORT}"
  exit 0
fi
tauri-driver --port "${PORT}" &
echo $! > /tmp/tauri-driver.pid
echo "[start-driver] tauri-driver started on ${PORT} (pid $(cat /tmp/tauri-driver.pid))"
```

配套 `stop-driver.sh`（kill pid + 清理）。

### 2.3 杠杆 3：tauri-plugin-wdio 注册（OQ-2 决策落盘，v3 勘误-3 补 Steps 3-6）

#### 2.3.1 实施方案（生产 Cargo.toml，6 步完整版）

> **v3 勘误-3（2026-06-20）**：T-09 实施时发现原 design 仅列 Step 1-2，实测 @wdio/tauri-service v1.1.0 的"Tauri plugin not available"告警 **不会因 Rust 注册而消除** —— 需要官方文档（[plugin-setup.md](https://github.com/webdriverio/desktop-mobile/blob/main/packages/tauri-service/docs/plugin-setup.md)）的完整 6 步。本节补齐。

**Step 1**：`src-tauri/Cargo.toml` 加依赖

```toml
[dependencies]
# ... 现有依赖
tauri-plugin-wdio = "^1"  # 版本与 @wdio/tauri-service v1.1.0 兼容
```

**Step 2**：`src-tauri/src/lib.rs` 注册插件

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_wdio::init())  // F115 新增
    .setup(|app| { /* ... */ })
    // ...
```

**Step 3**：`src-tauri/capabilities/default.json` 加 `wdio:default` 权限

```json
{
  "permissions": [
    "core:default",
    "opener:default",
    "wdio:default"
  ]
}
```

**Step 4**：`src-tauri/tauri.conf.json` 加 `withGlobalTauri: true`

> GoGuo 前端用 `@tauri-apps/api/core` 的 `invoke()` —— 该 wrapper 在 v2 默认通过 `window.__TAURI_INTERNALS__`（始终可用）调用，启用 `withGlobalTauri` 是**叠加**暴露 `window.__TAURI__`，对既有 IPC 无破坏。

**Step 5**：GoGuo 前端安装 + import `@wdio/tauri-plugin`

```bash
# 根目录
pnpm add @wdio/tauri-plugin
```

```typescript
// src/main.tsx
import '@wdio/tauri-plugin';  // 副作用 import：注册 window.wdioTauri
```

**Step 6**：`cargo build --release` + `npx vite build`，验证告警消除

**Step 7**：SC-8 全量回归（625 测试 + 手动冒烟）

#### 2.3.2 风险控制（OQ-6 阈值）

| 指标 | 阈值 | 度量方法 | 超阈回退 |
|------|------|---------|---------|
| 二进制体积 | 增长 ≤ 2 MB | `stat -c %s target/release/goguo` before/after | 降级为 dev-only feature gate（OQ-2 备选） |
| 冷启动时间 | 增长 ≤ 50 ms | `hyperfine --warmup 1 --runs 5 './target/release/goguo --version'` | 同上 |
| RSS 内存（idle 30s） | 增长 ≤ 5 MB | `/usr/bin/time -v` 采样 | 记录但不回退 |

详细取舍记录见 §10 ADR-0008。

#### 2.3.3 不可逆性评估

| 维度 | 评估 |
|------|------|
| 二进制兼容 | plugin 是 Tauri Builder 链的 `.plugin()` 注册，移除即恢复 |
| 数据兼容 | plugin 不写本地数据，无 schema 变更 |
| 配置兼容 | plugin 不读取 config，无配置迁移 |
| 前端兼容 | `withGlobalTauri=true` 是叠加而非替换 `__TAURI_INTERNALS__`；`import '@wdio/tauri-plugin'` 是副作用 import，移除即恢复 |
| 结论 | **可逆**——若 v0.2.0 发现问题，回滚 5 处变更（Cargo.toml / lib.rs / capabilities / tauri.conf.json / main.tsx + pnpm remove）即可恢复 |

### 2.4 优化效果度量（spec FR-2.2.4）

#### 2.4.1 度量矩阵

| 杠杆 | baseline | 实施后预期 | 验收阈值 | 度量时机 |
|------|---------|----------|---------|---------|
| 杠杆 1（restart:false） | 95s | ≤ 80s | ≤ 80s（5 次均值） | M4 |
| 杠杆 2（driver 常驻） | 上述 | 再降 ≥ 8s | ≤ 72s（5 次均值） | M4 |
| 杠杆 3（plugin 注册） | 同上 | 无耗时影响 | 仅消除警告 | M4 |
| **三杠杆合计（多 spec 场景）** | 95s | **≤ 70s（SC-2）** | ≤ 70s | M4 finalize |
| 单 spec 调试场景 | ≈ 40s | ≈ 40s（无变化） | 无需优化（restart:false 默认行为已支持） | — |

> **适用范围说明（TODO id:04 处理结果，2026-06-19，参见标注 id:01）**：
> 三杠杆合计 ≤ 70s 阈值**仅适用于多 spec 场景**（`pnpm test:feature` / `pnpm test:all` / `pnpm test:e2e`，跑 f114-baseline 下 5 spec 或跨 Feature 累积）。
> 单 spec 调试场景（`wdio run --spec <path>`）baseline ≈ 40s，杠杆 1 无收益但不亏损，不纳入 SC-2 验收范围。
> 度量脚本（§2.4.2）的 5 次连跑默认配置为多 spec 场景。
<!-- TODO id:04;status:close;date:2026-06-19T13:30 根据spec范围调整同步调整度量矩阵及后续设计，参见标注id:01；任务处理结果：联动 id:01 处理结论（保留杠杆 1）。§2.4.1 度量矩阵已补「单 spec 调试场景」行 + 「适用范围说明」段（明确 ≤ 70s 阈值仅适用多 spec 场景），不调整主体度量。§2.4.2 度量脚本默认配置不变（多 spec 场景）。 -->
#### 2.4.2 度量脚本设计

`e2e/scripts/benchmark.sh`（M4 实施）：

```bash
#!/usr/bin/env bash
# 跑 5 次 smoke + ipc，记录耗时，输出均值
RESULTS=()
for i in 1 2 3 4 5; do
  START=$(date +%s.%N)
  pnpm test:e2e 2>&1 | tail -5
  END=$(date +%s.%N)
  DUR=$(echo "${END} - ${START}" | bc)
  RESULTS+=("${DUR}")
done
echo "Runs: ${RESULTS[*]}"
# 计算均值（python/awk 任选）
```

度量结果记录到 `features/115-ux-e2e-infrastructure/evidence/benchmark-M4.md`（finalize 证据）。

---

## 3. 项目级 e2e 基础设施设计（spec FR-2.1）

### 3.1 目录结构（spec FR-2.1.1）

完整结构见 spec §2.1.1-R1。本节补充**实施顺序**与**迁移策略**：

#### 3.1.1 迁移策略（无回归保证）

| 步骤 | 操作 | 验证 |
|------|------|------|
| 1 | 创建 `e2e/specs/f114-baseline/` | 目录存在 |
| 2 | `git mv e2e/test/specs/smoke.spec.ts e2e/specs/f114-baseline/` | 文件移动 |
| 3 | `git mv e2e/test/specs/ipc.spec.ts e2e/specs/f114-baseline/` | 文件移动 |
| 4 | 删除 `e2e/test/` 空目录 | `ls e2e/test/` 失败 |
| 5 | 更新 `e2e/wdio.conf.ts` 的 `specs` glob：`./specs/**/*.spec.ts` | spec 跑通 |
| 6 | 跑 `pnpm test` 确认 smoke + ipc 全过 | 0 回归 |

#### 3.1.2 独立 npm 包（OQ-5）

保持 e2e/package.json 独立，理由见 spec §1.2 + C-I4。主仓库根 `package.json` 仅加转发入口（FR-2.1.3）。

### 3.2 helpers 抽取（spec FR-2.1.2）

#### 3.2.1 helper 清单与契约

| helper 文件 | 导出函数 | 契约 | L1 单测 |
|------------|---------|------|---------|
| `helpers/tauri-ipc.ts` | `invokeTauri<T>(cmd, args?)` | 走 `window.__TAURI_INTERNALS__.invoke` | 间接（被 spec 覆盖） |
| `helpers/wait.ts` | `waitForGoGuoReady(timeout?)` | `$("body").waitForExist({ timeout })` | 间接 |
| `helpers/env.ts` | `isWSL()` / `ensureX11Backend()` / `getTauriDriverPort()` / `shouldReuseDriver()` | 见下表 | **直接**（FR-2.2.4-R3 强制） |
| `helpers/state.ts`（新增） | `resetGoGuoState()` | 清理 SiteRulesState 等可变状态 | **直接**（杠杆 1 状态隔离逻辑） |

**`helpers/env.ts` 函数契约**（L1 单测覆盖边界）：

| 函数 | 行为 | 边界用例 |
|------|------|---------|
| `isWSL()` | 读 `/proc/version` grep `microsoft` | WSL / 原生 Linux / 文件不存在 |
| `ensureX11Backend()` | 设 `process.env.GDK_BACKEND = "x11"`（仅 WSL） | 已设 / 未设 / 非 WSL |
| `getTauriDriverPort()` | 读 `process.env.TAURI_DRIVER_PORT`，默认 4444 | 未设 / 空字符串 / 非数字 / 超出 1~65535 |
| `shouldReuseDriver()` | 读 `process.env.TAURI_DRIVER_REUSE === "1"` | 未设 / "0" / "1" / 其它值 |

#### 3.2.2 vitest in e2e/（OQ-7）

**目录**：`e2e/helpers/__tests__/*.test.ts`
**入口**：`e2e/package.json` 新增 `"test:unit": "vitest run"`
**依赖**：`e2e/package.json` 加 `vitest` 到 devDependencies
**覆盖率目标**：env.ts / state.ts ≥ 80%（FR-2.2.4-R3）

### 3.3 主仓库入口（spec FR-2.1.3）

#### 3.3.1 根 package.json scripts（R1~R3）

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "pnpm exec vitest run",
    "test:watch": "pnpm exec vitest",
    "test:e2e": "cd e2e && pnpm test",                              // R1
    "test:all": "pnpm test && pnpm test:e2e",                        // R2
    "test:feature": "node scripts/test-feature.mjs",                 // R4
    "test:e2e:feature": "cd e2e && pnpm test:e2e:feature"            // R5 转发
  }
}
```

#### 3.3.2 单 Feature 入口实现（R4~R7）

**`scripts/test-feature.mjs`**（根目录，新增）：

```javascript
#!/usr/bin/env node
// 接收 feature-id 参数，分三层执行：cargo test + vitest + e2e
import { spawnSync } from "node:child_process";

const featureId = process.argv[2];
if (!featureId) {
  console.error("Usage: pnpm test:feature -- <feature-id>");
  process.exit(1);
}

console.log(`[test:feature] Running tests for ${featureId}...`);

// L1+L2+L3: cargo test
const cargo = spawnSync("cargo", ["test", "--workspace", featureId], { stdio: "inherit" });

// L4: vitest
const vitest = spawnSync("pnpm", ["exec", "vitest", "run", "--", "--testNamePattern", featureId], { stdio: "inherit" });

// 汇总
console.log(`[test:feature] Summary:`);
console.log(`  cargo: ${cargo.status === 0 ? "PASS" : "FAIL"}`);
console.log(`  vitest: ${vitest.status === 0 ? "PASS" : "FAIL"}`);
process.exit(cargo.status ?? 1 | vitest.status ?? 1);
```

**`e2e/package.json` 转发**（R6）：

```json
{
  "scripts": {
    "test": "wdio run ./wdio.conf.ts",
    "test:headed": "GDK_BACKEND=x11 wdio run ./wdio.conf.ts",
    "test:unit": "vitest run",                                       // OQ-7
    "test:e2e:feature": "wdio run ./wdio.conf.ts --spec"             // R5 占位，实际拼接在 mjs
  }
}
```

> **R5 实现细节**：`pnpm test:e2e:feature -- f201` 等价于 `cd e2e && wdio run ./wdio.conf.ts --spec specs/f201-first-run/**/*.spec.ts`。在 `scripts/test-feature.mjs` 中追加 e2e 层（若 `e2e/specs/<feature-id>/` 目录存在）。

#### 3.3.3 R7 自动补全降级

`pnpm test:feature -- <Tab>` 在 pnpm 下不支持自动补全（需 shell 集成）。**降级方案**：在 `e2e/README.md` 列出可用 feature-id 清单（扫描 `e2e/specs/` + `features/`），用户手动传参。满足 R7 后半句"或 README 清单"。

---

## 4. 测试等级矩阵 L1~L5 设计（spec FR-2.3）

### 4.1 文档结构（OQ-1）

**位置**：`docs/test-level-matrix.md`（独立，与 F113 `docs/test-trace-matrix.md` 并存）

**双向链接**：
- test-level-matrix.md 顶部声明"FR 测试函数追溯见 [test-trace-matrix.md](./test-trace-matrix.md)"
- test-trace-matrix.md 顶部声明"能力的 L1~L5 分工见 [test-level-matrix.md](./test-level-matrix.md)"

**章节结构**：

```markdown
# 测试等级矩阵（L1~L5）

> 维护时机：每个 Feature 在 hf-specify 阶段占行；hf-design 阶段填函数名；hf-finalize 阶段验证。
> 与 docs/test-trace-matrix.md（FR→测试函数 1:1 追溯）并存。

## 1. 等级决策原则（spec FR-2.3.1-R4）
[8 条原则表]

## 2. 矩阵
### 2.1 F201: 首次安装引导与基线确认闭环
[9 行表]

### 2.2 F202~F205
<TBD by 各 Feature design 阶段>

## 3. 矩阵更新规则
- specify 阶段：占行 + 等级标注
- design 阶段：填测试函数名
- finalize 阶段：grep <TBD 计数 = 0
```

### 4.2 等级决策原则（spec FR-2.3.1-R4 落盘）

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

### 4.3 F201 首案例填充（spec FR-2.3.1-R3 / R3a 阶段 1）

> **填充策略**：F115 阶段为每条 FR 占行 + 等级标注；L2 列对**继承自 F001~F004 的测试**填具体函数名（来自 trace-matrix），对**全新增的 FR** 标 `<TBD by F201 design>`；其它列若 F201 design 未启动，等级标注到位即可。

#### 4.3.1 F201 矩阵（9 FR，F115 finalize 时结构完整）

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

#### 4.3.2 阶段 1 完整性自检（spec FR-2.3.1-R3a）

| 指标 | 要求 | 实际 | 结果 |
|------|------|------|------|
| 行数 | ≥ F201 spec FR 总数 × 0.6 = 9 × 0.6 = 5.4 | 9 行 | ✅ |
| 等级标注完整 | 每行 L1~L5 列至少有等级标注 | 9 行全标注 | ✅ |
| `<TBD` 计数 | ≤ 9 × 0.4 = 3.6 | 见下表分析 | ⚠️ 详 §4.3.3 |

#### 4.3.3 `<TBD` 计数与阈值差距分析

逐行统计 `<TBD` 出现：
- FR-1.0: 0
- FR-1.1: 2（L4/L5）
- FR-1.2: 5（L1/L2/L4/L5 + AdjustmentItem 列表）
- FR-1.3: 4（L1/L3/L4/L5）
- FR-1.4: 2（L4/L5）
- FR-1.5: 4（L1/L2/L4/L5）
- FR-1.6: 2（L4/L5）
- FR-1.7: 4（L1/L2/L4 + 守卫）
- FR-1.8: 5（L1/L2/L3/L4/L5）
- **合计 ≈ 28**

**vs 阈值 3.6**：远超。

**根因**：F115 阶段无法为 F201 的 L1/L4/L5 填具体函数名——F201 design 阶段未启动（C-P4 隔离）。L2 列对继承测试填了具体函数名（6/9），对全新增 3 个 FR（FR-1.2/1.7/1.8）填 TBD（3 个）。

**修订 spec FR-2.3.1-R3a 阈值解读**：spec 原文"矩阵 grep `<TBD` 计数 ≤ (F201 行数 × 0.4)"在 F115 阶段不可达。修订为**两阶段分别考核**：

| 阶段 | TBD 阈值（修订建议） | 当前实际 |
|------|--------------------|---------|
| F115 finalize（结构完整） | **L2 列** `<TBD` ≤ 行数 × 0.4 = 3.6（即 L2 列 ≥ 60% 有继承函数名） | 3 个（FR-1.2/1.7/1.8），达标 |
| F201 finalize（测试代码完整） | **全表** `<TBD` 计数 = 0 | 待 F201 |

> **design 决策**：F115 finalize 时仅考核 L2 列 TBD ≤ 3.6（已达成：3 个）；其它列 TBD 推到 F201 design 阶段填入。design review 时若 reviewer 认可此修订，回写 spec FR-2.3.1-R3a 补充说明。

### 4.4 矩阵执行约束（spec FR-2.3.2）

- L4 能力不重复在 e2e/ 实现（避免冗余）
- L5 能力必须有 e2e spec 承接
- `docs/principles/testing-principles.md` 新增"L1~L5 等级决策原则"小节（tasks 阶段落地）

---

## 5. Feature 接入规范设计（spec FR-2.4）

### 5.1 AGENTS.md §7 + testing-principles.md 双文档落盘（spec FR-2.4.3）

> **结构说明（? id:05 处理结果，2026-06-19）**：
> 采纳 reviewer 建议——AGENTS.md §7 仅作**入口 + 摘要**（关注点分离，避免 Agent 注入上下文过长）；详细条款（HF 全流程检查点表、§N 模板引用、豁免清单说明）落盘到 `docs/principles/testing-principles.md`。
> 两个文档由 F115 tasks 阶段（FR-2.4.3-R4）一次 commit 落盘：`docs(agents+testing-principles): enforce L1~L5 test design section per feature`。

#### 5.1.1 AGENTS.md §7 实际落盘版本（精简，入口 + 摘要）

```markdown
## 7. Feature 自动化测试设计强制规范

> 详细条款、HF 全流程检查点、豁免清单说明见 `docs/principles/testing-principles.md` §"L1~L5 自动化测试设计强制规范"。
> 立项日期：2026-06-18（F115 建立）；适用范围：所有新立项 Feature 与新启动的 fix Feature。

### 7.1 强制条款（摘要）

- **条款 1**：`hf-design` 必须输出 `§N L1~L5 自动化测试设计` 章节（模板：`docs/principles/test-design-section-template.md`），作为 design.md 必填章节。
- **条款 2**：该章节未通过 review **不得进入** `hf-tasks` 阶段。
- **条款 3**：章节必须同时覆盖 **UX 能力（L4/L5）** 与 **非 UI 能力（L1/L2/L3）**，每条 FR 给出测试用例 + 数据 + 脚本入口。
- **条款 4**：`hf-finalize` 必须验证该章节中所有声明的测试已实现且通过，否则不通过完成门。

### 7.2 引用关系

- 强制规范详细条款：`docs/principles/testing-principles.md` §"L1~L5 自动化测试设计强制规范"
- 章节模板：`docs/principles/test-design-section-template.md`
- 测试等级矩阵：`docs/test-level-matrix.md`
- §4 "Coding / Testing / Architecture 标准" 交叉引用本节
```

#### 5.1.2 testing-principles.md 新增小节（详细条款落盘）

> 落盘位置：`docs/principles/testing-principles.md` 末尾新增 `## L1~L5 自动化测试设计强制规范` 小节。
> 落盘 commit 与 AGENTS.md §7 同一次提交。

```markdown
## L1~L5 自动化测试设计强制规范

> 来源：F115（2026-06-18 立项）。AGENTS.md §7 入口指向本节。

### 强制条款（与 AGENTS.md §7.1 摘要一致，详见 AGENTS.md）

### HF 全流程检查点

| HF 阶段 | 检查点 | 责任人 |
|---------|-------|-------|
| `hf-specify` | 矩阵为本 Feature 占行（FR ID + 等级标注，函数名允许 TBD） | spec 作者 |
| `hf-design` | design.md 含完整 §N "L1~L5 自动化测试设计"章节；矩阵函数名填齐；测试用例 + 数据 + 脚本设计完成 | design 作者 |
| `hf-tasks` | tasks.md 拆解时含每条测试的实施 task（按 §N.5 顺序） | tasks 作者 |
| `hf-test-driven-dev` | 按 §N.5 RED-GREEN 执行；不允许跳过 L1~L5 任意层 | 实施者 |
| `hf-finalize` | 验证 §N 中所有声明的测试已实现且通过；矩阵 TBD 计数 = 0 | finalize 审查 |

### 显式豁免清单（不要求补 §N 章节）

F109、F110、F114、F115、F101~F106（已立项但未启动项启动时按"新启动 fix Feature"对待）

### L1~L5 等级决策原则

[引用本文件已有的等级决策原则小节，由 §4.2 落盘]
```

**配套改动**（tasks 阶段）：
- AGENTS.md §4 末尾加一行：`测试设计强制规范：§7（所有新 Feature 必填"L1~L5 自动化测试设计"章节，详细条款见 testing-principles.md）`
- AGENTS.md §6 末尾或 §7 末尾加一行：`仓库根 README.md "Active feature 指针来源"附近新增提示文案`（FR-2.4.3-R3）
<!-- ? id:05;status:close;date:2026-06-19T13:30 建议在AGENTS.md中仅提供文档入口及简介说明，具体的接入规范条款，纳入docs/principles/testing-principles.md文档中；任务处理结果：采纳折中版——AGENTS.md §7 精简为入口 + 4 条强制条款摘要 + 引用关系；HF 全流程检查点表 + 豁免清单详细说明 + 等级原则落盘到 testing-principles.md 新增小节。两文档同 commit 落盘。§5.1 已重构为 §5.1.1（AGENTS.md §7 精简版）+ §5.1.2（testing-principles.md 详细条款） -->

### 5.2 §N 章节模板最终稿（spec FR-2.4.4）

> 本节为 `docs/principles/test-design-section-template.md` 的**内容草稿**，tasks 阶段（FR-2.4.4-R2）落盘。

```markdown
# design.md "L1~L5 自动化测试设计"章节模板

> 强制章节（AGENTS.md §7）。本章在编码启动前完成，覆盖本 Feature 全部 FR。
> 引用：`docs/test-level-matrix.md`（等级分工）、`docs/principles/testing-principles.md`（等级原则）。

## 阈值与简化规则

| 触发条件 | 模板版本 | 必填子节 | 可选子节 |
|---------|---------|---------|---------|
| FR 总数 > 5 | **完整版**（下方结构） | N.1 / N.2.1~N.2.5 / N.5 | N.3 / N.4 |
| FR 总数 ≤ 5 | **简化版**（单表） | 单表（含 FR ID/能力/L1~L5/关联 FR） | N.3 / N.4 / N.5 |

---

## 完整版结构（FR > 5）

### N.1 测试等级矩阵填充
- 列出本 Feature 在 `docs/test-level-matrix.md` 中新增的行（至少含 FR ID + 等级标注）
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

#### N.2.4 L4（vitest + RTL，UX 用例写法）
| spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR |
|---------|---------|---------|---------|--------|
| ... | 1 | ...（用户动作） | ...（可观测结果） | ... |

> **UX 写法约束（? id:06 处理结果）**：L4/L5 涉及前端用户交互的用例必须使用通用 UX 测试用例写法（操作序号 + 操作描述 + 期望结果），不局限于"describe/it + 渲染场景"。

#### N.2.5 L5（e2e spec，UX 用例写法）
| spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR |
|---------|---------|---------|---------|--------|
| e2e/specs/<feature-id>/... | 1 | ...（用户动作） | ...（可观测结果） | ... |

### N.3 测试数据（可选）
- 共享 fixtures（`e2e/fixtures/`）vs Feature 私有 fixtures（`features/<NNN>/fixtures/`）
- 测试用 site_id / mock 数据清单

### N.4 测试脚本入口（可选）
- 单 Feature 全量测试：`pnpm test:feature -- <id>`
- 单 Feature e2e：`pnpm test:e2e:feature -- <id>`
- 全套：`pnpm test:all`

### N.5 TDD 执行顺序（必填）
- 列出 RED → GREEN → REFACTOR 的实施顺序（按 FR 优先级）

---

## 简化版结构（FR ≤ 5）

### N. L1~L5 自动化测试设计（简化版，FR ≤ 5）

| FR ID | 能力 | L1 | L2 | L3 | L4 | L5 | 关联 FR | 依据 |
|-------|------|----|----|----|----|----|--------|------|
| ... | ... | 函数名/N/A | 函数名/N/A | 函数名/N/A | spec:it/N/A | spec:it/N/A | ... | ... |

附（可选）：测试数据 / 脚本入口 / TDD 顺序（按需附加）
```
<!-- ? id:06;status:close;date:2026-06-19T13:30 对于L4/L5涉及前端用户交互的测试用例，使用通用UX测试用例的写法，可能包括但不限于操作序号、操作描述、期望结果；任务处理结果：采纳。§5.2 §N 模板 L4/L5 子节列结构已改为「spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR」（原「describe/it + 渲染场景」退役）。F115 自身 §N.2.4（L4 不适用，F115 无前端组件改动）+ §N.2.5（L5 现有 5 spec 待 tasks 阶段按新列结构填充）同步适用。spec.md FR-2.4.4-R2 待回写（补「L4/L5 用例需含操作步骤 + 期望结果」约束）。同时修复全角问号格式错误。 -->

### 5.3 接入规范 lint 脚本（spec FR-2.4.2）

**`e2e/scripts/lint-specs.mjs`**（新增）：

```javascript
#!/usr/bin/env node
// 校验：所有 spec 在 e2e/specs/f<NNN>-<slug>/ 下、describe 含 Feature ID、helper import 来自 e2e/helpers/
import { globSync } from "node:fs";
import { readFileSync } from "node:fs";

const specs = globSync("e2e/specs/**/*.spec.ts");
let errors = [];

for (const spec of specs) {
  const content = readFileSync(spec, "utf-8");
  const match = spec.match(/specs\/(f\d{3}-[\w-]+)\//);
  if (!match) errors.push(`${spec}: 不在 f<NNN>-<slug>/ 目录下`);
  const featureId = match?.[1];
  if (!content.includes(`describe("${featureId}`) && !content.includes(`describe('${featureId}`)) {
    errors.push(`${spec}: describe 不含 Feature ID "${featureId}"`);
  }
  if (content.includes("__TAURI_INTERNALS__")) {
    errors.push(`${spec}: 直接使用 __TAURI_INTERNALS__，应改为 import from "../../helpers/tauri-ipc"`);
  }
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`[lint-specs] ${specs.length} specs OK`);
```

**入口**：`e2e/package.json` 加 `"lint": "node scripts/lint-specs.mjs"`

---

## 6. 开发环境配置文档化设计（spec FR-2.5）

### 6.1 setup-dev-env.sh 设计（spec FR-2.5.2-R2）

**`e2e/scripts/setup-dev-env.sh`**：

```bash
#!/usr/bin/env bash
# F115: 开发环境首次配置（镜像绕过方案）
# 处理 GAP-F115-2 的开发态缓解（根因修复推到 F116+）
set -euo pipefail

CARGO_CONFIG="$HOME/.cargo/config.toml"
E2E_NPMRC="$(dirname "$0")/../.npmrc"

echo "[setup-dev-env] 检查 cargo 镜像配置..."

# 1. cargo: rsproxy-sparse 镜像
if [ ! -f "$CARGO_CONFIG" ] || ! grep -q "rsproxy-sparse" "$CARGO_CONFIG"; then
  mkdir -p "$(dirname "$CARGO_CONFIG")"
  cat >> "$CARGO_CONFIG" <<'EOF'

# F115 setup-dev-env.sh 自动添加（rsproxy 镜像，绕过 mihomo 阻断）
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[source.crates-io]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "sparse+https://rsproxy.cn/index/"
EOF
  echo "[setup-dev-env] cargo 镜像已配置（rsproxy-sparse）"
else
  echo "[setup-dev-env] cargo 镜像已存在，跳过"
fi

# 2. e2e/.npmrc 校验（C-I4 强制项）
if [ ! -f "$E2E_NPMRC" ]; then
  echo "[setup-dev-env] ERROR: $E2E_NPMRC 不存在（C-I4 镜像隔离策略必需）"
  exit 1
fi
if ! grep -q "npmmirror" "$E2E_NPMRC"; then
  echo "[setup-dev-env] WARNING: $E2E_NPMRC 未含 npmmirror 配置，请检查"
fi
echo "[setup-dev-env] e2e/.npmrc 校验通过"

# 3. 验证 cargo 可达
echo "[setup-dev-env] 验证 cargo install --dry-run..."
if cargo install tauri-driver --dry-run 2>&1 | grep -qi "SSL_ERROR\|network"; then
  echo "[setup-dev-env] WARNING: cargo 仍报网络错误，请检查 mihomo 状态或手动配置"
else
  echo "[setup-dev-env] cargo 网络正常"
fi

echo "[setup-dev-env] 配置完成。可运行 'pnpm test:e2e'。"
```

**幂等保证**：脚本通过 `grep` 检查配置是否已存在，重复执行不产生重复写入。

### 6.2 e2e/README.md 结构（spec FR-2.5.2-R1/R3）

新增三个章节（在现有 README 基础上）：

```markdown
## Step 0: 开发环境首次配置（必读）

> F115 引入。处理 GAP-F115-2（mihomo config 阻断 cargo/pnpm 流量）的开发态缓解。
> 根因修复推到 F116+（详见 F110 §12 GAP-F115-2）。

首次配置：
```bash
cd e2e && bash scripts/setup-dev-env.sh
```

或手动配置：
- `~/.cargo/config.toml` 加 rsproxy-sparse 镜像源
- `e2e/.npmrc` 含 npmmirror.com（C-I4 隔离策略）

## Feature 接入流程

[spec FR-2.4.1-R1 的 Step 1~5 内容]

## 已知限制

> F115 引入。多实例共存场景下的已知问题，**不在 F115 范围内修复**。

| 限制 | 严重度 | 描述 | 移交 |
|------|-------|------|------|
| `/etc/environment` 多实例覆盖 | 🔴 HIGH | `~/apps/goguo` 与 `target/release/goguo` 同时运行时，write_state 互相覆盖 | F110 §12 GAP-F115-1（建议 F116+） |
| mihomo config 阻断 cargo/pnpm | 🟡 MED | site-crates / site-npmjs ruleset 无 DIRECT 规则，依赖镜像绕过 | F110 §12 GAP-F115-2 |
| mihomo config dev/prod 拆分 | 🟢 LOW | 单一 config 文件耦合开发态与生产态规则 | F110 §12 GAP-F115-3 |
```

### 6.3 默认不引入 P3（HOME 隔离）的依据（spec FR-2.5.2-R4）

**评估结论**：基于代码事实（`mihomo_manager.rs:144-150` adopt 机制），多实例 mihomo 自洽，P3 无价值。

| 维度 | P3 价值评估 |
|------|------------|
| 隔离 mihomo 进程 | ❌ adopt 已自洽 |
| 隔离状态文件 | ⚠️ 部分有效，但 e2e 阶段状态隔离已由 `resetGoGuoState()` 处理（杠杆 1） |
| 隔离 /etc/environment | ❌ HOME 无效（路径硬编码） |
| 实施成本 | MED（wdio.conf.ts 注入 HOME + 测试 HOME 隔离是否生效） |
| 结论 | **不引入**。若 GAP-F115-1 修复（路径参数化）后再评估 |

---

## 7. 网络/环境隔离方案评估（OQ-3 决策落盘）

### 7.1 P1~P4 评估与排除依据

| 方案 | 描述 | 排除依据 |
|------|------|---------|
| **P1** mihomo config 拆 dev/test/prod | 按 GOGUO_PROFILE 切换 config | 改动 release 流程，影响生产用户；属 GAP-F115-3 范畴，推 F116+ |
| **P2** 新增 site-dev-tools ruleset | 放行 cargo/pnpm 域名 | 改 release/data/mihomo/config.yaml（违反 C-I5）；属 GAP-F115-2 范畴 |
| **P3** HOME 隔离 | e2e 启动 GoGuo 时独立 HOME | 代码事实：adopt 已自洽 mihomo；/etc/environment 路径硬编码 HOME 无效；详见 §6.3 |
| **P4** 容器化 | Docker/Podman 完全隔离 | OQ-4 已否；与"不引入 CI/CD"基调冲突；WSL2 内跑容器额外依赖 |

### 7.2 多实例行为代码事实（调研证据）

| 维度 | 代码事实 | 来源 |
|------|---------|------|
| Tauri 单实例锁 | ❌ 未注册 | `src-tauri/Cargo.toml` grep 无匹配 |
| mihomo adopt 机制 | 第二实例发现 9090 已响应 → 复用已有 mihomo | `mihomo_manager.rs:144-150` |
| mihomo 端口 | 固定 mixed-port=7890/socks-port=7891/controller=9090 | `release/data/mihomo/config.yaml:6-8` |
| install_root | `get_install_root()` 决定，data_dir = install_root/data | `lib.rs:32-60` |
| /etc/environment 路径 | 硬编码 `Path::new("/etc/environment")` | `wsl.rs:255`、`linux.rs:27` |

### 7.3 GAP 移交确认

F115 调研发现的 3 个 GAP 已同步落盘到：
- `features/110-design-gap-closure/design.md` §12（详细描述 + 代码事实 + 修复建议）
- `docs/insights/F001-F004-GAP-Analyses/feature-restructure-e2e-loops.md` §9（索引表 + 与 Feature 关系）

F115 不修复，仅在 `e2e/README.md` "已知限制"节声明 + `setup-dev-env.sh` 文档化 GAP-F115-2 的开发态缓解。

---

## 8. 风险与缓解（spec §5 细化）

> 复用 spec §5 风险表，本节补充 design 阶段细化的缓解措施。

| # | 风险 | spec 等级 | design 阶段细化缓解 |
|---|------|---------|-------------------|
| R1 | tauri-plugin-wdio 破坏现有功能 | HIGH | OQ-6 阈值 + 超阈回退 dev-only gate（§2.3.2）+ SC-8 全量回归 |
| R2 | restart:false 状态污染 | MED | `resetGoGuoState()` helper（§2.1.3）+ L1 单测覆盖边界 |
| R3 | driver 复用模式端口占用 | MED | start-driver.sh + stop-driver.sh + pid 文件管理 |
| R4 | 矩阵过严导致返工 | MED | §N 简化版阈值（FR ≤ 5）+ 矩阵"指南"定位 |
| R6 | mihomo 阻断开发态流量 | HIGH | setup-dev-env.sh 一键脚本 + 文档化（§6.1）；根治推 F116+ |
| R7 | F201 接入验证失败 | LOW | F201 design 阶段才验证（C-P4）；F115 仅自验证 §N |
| R9 | AGENTS §7 过严 | MED | 显式豁免清单（§5.1） |
| R10 | §N 模板负担过重 | MED | 简化版阈值（§5.2） |
| R11 | /etc/environment 多实例覆盖 | HIGH | GAP-F115-1 移交 F116+（不在 F115 解决） |

---

## 9. 依赖与约束

### 9.1 依赖版本矩阵

| 依赖 | 当前版本 | F115 要求 | 来源 |
|------|---------|----------|------|
| tauri-driver | v2.0.6（~/.cargo） | 不变 | F114 PoC |
| @wdio/tauri-service | v1.1.0 | 不变 | F114 PoC |
| @wdio/cli / webdriverio | v9.x | 不变 | F114 PoC |
| tauri-plugin-wdio | TBD | 与 @wdio/tauri-service v1.1.0 兼容 | F115 新增 |
| vitest | v3.x（主仓库） | 引入到 e2e/（OQ-7） | F115 新增 |

### 9.2 约束清单（spec §4 摘要）

- C-I1~I5：F114 PoC 关键配置 + 镜像隔离 + 不改 mihomo config
- C-T1~T5：tauri-plugin-wdio 兼容、不引入 Selenium/Playwright/CI/CD
- C-P1~P5：interactive 流程、本地验证、dev-only gate 评估、F201 隔离、workspace clean

---

## 10. ADR-0008 草稿：生产 Cargo.toml 引入 tauri-plugin-wdio

> 本节为 `docs/adr/0008-tauri-plugin-wdio-in-production-cargo-toml.md` 的**内容草稿**。
> 编号说明：ADR-0001~0007 已占用（ADR-0007 为 F002 协同模式远程适配器），本决策使用 **ADR-0008**。
>
> **落盘策略（决策 B，2026-06-18）**：design 阶段不创建 `docs/adr/0008-*.md` 文件；待 tasks 阶段实测 §10.4 取舍表中的量化数据（二进制体积 / 冷启动 / 优化后耗时）回填草稿后，于 TDD §N.5 #17 一次性落盘最终版。design 与 tasks 之间的 trace 通过本节草稿 + §13 评审点 #7 + §N.5 #17 + §12 检查清单维持。

```markdown
# ADR-0008：生产 Cargo.toml 引入 tauri-plugin-wdio

- 状态：拟接受（待 F115 finalize 落盘）
- 日期：2026-06-18

## 背景

F114 PoC 验证 tauri-driver + WebDriverIO + @wdio/tauri-service 可在 WSL2 下建立桌面 E2E 基础设施。
@wdio/tauri-service 依赖 `tauri-plugin-wdio`（Tauri 插件）提供 window 状态查询与 mock 注入。
PoC 阶段未注册该插件，wdio 运行日志出现 "Tauri plugin not available. Make sure @wdio/tauri-plugin is installed" 警告（F114 PoC 报告 §5 风险登记）。

F115 spec FR-2.2.3 要求消除该警告，OQ-2 决策"进生产 Cargo.toml"。

## 决策

将 `tauri-plugin-wdio` 加入 `src-tauri/Cargo.toml` `[dependencies]`，并在 `src-tauri/src/lib.rs` `tauri::Builder` 链中注册 `.plugin(tauri_plugin_wdio::init())`。

**全 profile 启用**（不使用 dev-only feature gate）。

## 备选方案

| 方案 | 优势 | 劣势 | 决策 |
|------|------|------|------|
| **生产 Cargo.toml（全 profile）** | 单一构建产物；与 @wdio/tauri-service 默认路径一致 | 二进制体积/启动时间微增；攻击面微增 | **采纳** |
| dev-only feature gate | 生产二进制纯净 | 双构建矩阵复杂化；F114 PoC 已在 release 上验证可行，gate 化属于倒退 | 否决（OQ-2） |

## 取舍

| 维度 | 影响 | 控制措施 |
|------|------|---------|
| 二进制体积 | 增长 ≤ 2 MB（OQ-6 阈值） | tasks 阶段实测；超阈回退 dev-only |
| 冷启动时间 | 增长 ≤ 50 ms | 同上 |
| 攻击面 | plugin 提供的 IPC 命令对前端可见 | plugin 仅暴露 window 状态查询，无敏感操作 |
| 可逆性 | 移除两行（Cargo.toml + lib.rs）即恢复 | 高 |

## 影响

- F115 SC-3 / SC-8 验收依赖此决策
- 后续所有 e2e 测试默认假设 plugin 已注册
- 若未来版本需要移除（如 v0.5.0 重构测试基础设施），需评估 e2e spec 影响范围
```

---

## N. L1~L5 自动化测试设计（F115 自身模板自验证）

> 强制章节（AGENTS.md §7）。本章按 `docs/principles/test-design-section-template.md` 模板填写，作为后续 Feature 的参照样本。
> **特殊性**：F115 自身是基础设施 Feature，FR 总数 > 5（共 22 个 FR-2.X.Y 主条），使用完整版结构。

### N.1 测试等级矩阵填充

F115 自身的能力在矩阵中按维度占行（粗粒度）：

| 能力 ID | 能力描述 | 关联 FR | L1 | L2 | L3 | L4 | L5 | 依据 |
|---------|---------|--------|----|----|----|----|----|------|
| F115-INFRA-dir | e2e/ 目录规范化 | FR-2.1.1 | N/A（目录结构） | N/A | N/A | N/A | `f114-baseline/smoke.spec.ts`（沿用 PoC） | 结构变更，L5 跑通即验证 |
| F115-INFRA-helpers | helpers 抽取 | FR-2.1.2 | `env.test.ts` / `state.test.ts`（OQ-7） | N/A | N/A | N/A | N/A（helper 在 spec 间接覆盖） | helper 自身需 L1 |
| F115-INFRA-entry | 主仓库入口（test:feature 等） | FR-2.1.3 | N/A（脚本） | N/A | N/A | N/A | `f114-baseline/smoke+ipc.spec.ts` 跑通即验证 | 入口脚本，L5 端到端 |
| F115-OPT-session | cross-spec session 复用 | FR-2.2.1 | N/A | N/A | N/A | N/A | `smoke+ipc.spec.ts` 合跑 ≤ 80s | 端到端耗时验证 |
| F115-OPT-driver | tauri-driver 常驻 | FR-2.2.2 | `env.test.ts`：`shouldReuseDriver` 边界 | N/A | N/A | N/A | 复用模式 vs 自启模式对比 | helper L1 + 实测 L5 |
| F115-OPT-plugin | tauri-plugin-wdio 注册 | FR-2.2.3 | `lib.rs` 注册分支（若用 feature gate） | N/A | N/A | N/A | wdio 日志无警告 | 注册逻辑 L1（若有分支）+ 警告消失 L5 |
| F115-MATRIX | 测试等级矩阵 | FR-2.3 | N/A（文档） | N/A | N/A | N/A | N/A | 纯文档交付，由 F201 接入验证 |
| F115-AGENTS | AGENTS.md §7 | FR-2.4.3 | N/A（文档） | N/A | N/A | N/A | N/A | 纯文档，由 F201 接入验证 |
| F115-TEMPLATE | §N 模板 | FR-2.4.4 | N/A（文档） | N/A | N/A | N/A | **本节**即自验证 | F115 design.md §N 存在即验证 |
| F115-DEVENV | 开发环境配置文档化 | FR-2.5 | N/A（脚本） | N/A | N/A | N/A | setup-dev-env.sh 跑通即验证 | 脚本功能性，L5 验证 |

### N.2 测试用例设计（逐层）

#### N.2.1 L1（Rust/TS 单元测试）

| 测试函数/文件 | 模块 | 断言 | 覆盖率目标 |
|--------------|------|------|----------|
| `e2e/helpers/__tests__/env.test.ts > isWSL` | `helpers/env.ts` | WSL/原生 Linux/文件不存在三分支 | ≥ 80% |
| `e2e/helpers/__tests__/env.test.ts > ensureX11Backend` | 同上 | 已设/未设/非 WSL 三分支 | ≥ 80% |
| `e2e/helpers/__tests__/env.test.ts > getTauriDriverPort` | 同上 | 未设/空/非数字/超范围 + 默认 4444 | ≥ 80% |
| `e2e/helpers/__tests__/env.test.ts > shouldReuseDriver` | 同上 | 未设/"0"/"1"/其它值 | ≥ 80% |
| `e2e/helpers/__tests__/state.test.ts > resetGoGuoState` | `helpers/state.ts` | 空状态/单站点/多站点清理后为空 | ≥ 80% |
| `src-tauri/src/lib.rs` 单测（若 plugin 用 feature gate） | `lib.rs` | feature 启用/禁用两分支注册正确 | 100%（分支） |

#### N.2.2 L2（FR 验收测试）

F115 是基础设施 Feature，**多数 FR 不适合 L2**（L2 验证 spec FR 可观测结果，而 F115 的"可观测结果"多为文档存在性/脚本执行性）。下表仅列适合 L2 的：

| 测试函数 | 文件路径 | 可观测结果 | 关联 FR |
|---------|---------|----------|--------|
| N/A | — | — | — |

> **说明**：F115 的"可观测结果"通过 §N.2.5 L5 + finalize 证据清单（spec §9）验证，不走 L2。

#### N.2.3 L3（契约/管道测试）

| 测试函数 | 类型 | 关键断言 | 关联 FR |
|---------|------|---------|--------|
| N/A | — | — | — |

> F115 无 Rust trait 契约或跨模块管道，L3 不适用。

#### N.2.4 L4（vitest + RTL）

| spec 文件 | describe/it | 渲染场景 | 关联 FR |
|---------|-----------|---------|--------|
| N/A | — | — | — |

> F115 不改前端组件，L4 不适用。

#### N.2.5 L5（e2e spec，UX 用例写法，id:06 列结构）

| spec 文件 | 操作序号 | 操作描述 | 期望结果 | 关联 FR |
|---------|---------|---------|---------|--------|
| `smoke.spec.ts`（迁移自 PoC） | 1 | 启动 GoGuo，等待 body 渲染 | body.getText() 非空 | FR-2.1.1-R2 |
| `smoke.spec.ts` | 2 | 读取 browser.getTitle() | title 匹配 /GoGuo\|Tauri/i | FR-2.1.1-R2 |
| `ipc.spec.ts`（迁移自 PoC） | 1 | resetGoGuoState() 清理 → invoke('add_target_site', {siteId:'github'}) | resp.success=true 且 resp.site.id='github' | FR-2.1.2-R1 |
| `ipc.spec.ts` | 2 | 重复 invoke('add_target_site', {siteId:'github'}) | success=true（幂等）或 error 字段非空（明确错误） | FR-2.1.2-R1 |
| `session-reuse.spec.ts`（F115 新增） | 1 | 在 spec 内读取 `browser.sessionId` | sessionId 是非空 UUID 字符串（证明 session 已建立） | FR-2.2.1-R1 |
| `session-reuse.spec.ts` | 2 | 在同 spec 内调用 `invoke('list_target_sites')` | 返回数组（无需新建 session 即可调 IPC，证明 session 稳定） | FR-2.2.1-R1 |
| `session-reuse.spec.ts` | 3 | 合跑结束后查 wdio 输出（T-12 benchmark 脚本验） | 全跑日志 `Session ID:` 出现次数 = 1（vs 不复用时的 2） | FR-2.2.1-R2 |
| `driver-reuse.spec.ts`（F115 新增） | 1 | 检测 `process.env.TAURI_DRIVER_REUSE` | 若非 "1" 则 `this.skip()` 跳过，避免污染自启模式 | FR-2.2.2-R1 |
| `driver-reuse.spec.ts` | 2 | 复用模式下 `browser.sessionId` | 非空字符串（证明连到外部 tauri-driver） | FR-2.2.2-R1 |
| `driver-reuse.spec.ts` | 3 | 同 spec 内 invoke('add_target_site', {siteId:'github'}) | success=true（复用模式下 IPC 路径完整） | FR-2.2.2-R1 |
| `plugin-registered.spec.ts`（F115 新增） | 1 | `browser.execute(() => typeof window.wdioTauri)` | 返回 "object"（证明 plugin 副作用 import 生效） | FR-2.2.3-R3 |
| `plugin-registered.spec.ts` | 2 | `browser.execute(() => typeof window.wdioTauri.execute)` | 返回 "function"（Execute API 注册） | FR-2.2.3-R3 |
| `plugin-registered.spec.ts` | 3 | wdio 输出全文 grep `"Tauri plugin not available"` | 计数 = 0（T-09 6 步注册后告警消除） | FR-2.2.3-R3 |

### N.3 测试数据

- 共享 fixtures：`e2e/fixtures/sites.ts`（站点测试数据，沿用 PoC）
- Feature 私有：F115 无私有 fixtures（基础设施 Feature）
- 测试用 site_id：`github`（沿用 PoC ipc.spec.ts）

### N.4 测试脚本入口

| 入口 | 命令 | 用途 |
|------|------|------|
| 全套 | `pnpm test:all` | 主仓库 + e2e |
| 单 Feature 全量 | `pnpm test:feature -- f114-baseline` | F114 baseline 三层 |
| 单 Feature e2e | `pnpm test:e2e:feature -- f114-baseline` | F114 e2e |
| L1 单测（e2e helpers） | `pnpm --filter e2e test:unit` | env/state helper 单测 |
| L1 单测（Rust） | `cargo test --lib` | tauri-plugin-wdio 注册分支 |
| 97s benchmark | `bash e2e/scripts/benchmark.sh` | 优化效果度量 |
| 开发环境配置 | `bash e2e/scripts/setup-dev-env.sh` | 首次配置 |
| 接入规范 lint | `pnpm --filter e2e lint` | spec 命名/import 校验 |

### N.5 TDD 执行顺序

按"先无回归 → 再优化 → 最后侵入"顺序：

| 序号 | RED 起点 | GREEN 实现 | REFACTOR | 关联 FR |
|------|---------|----------|----------|--------|
| 1 | `e2e/test/` 目录存在 | 迁移到 `e2e/specs/f114-baseline/`，wdio.conf 更新 glob | — | FR-2.1.1-R2 |
| 2 | spec inline `__TAURI_INTERNALS__` | 抽取 `helpers/tauri-ipc.ts`，spec 改 import | env/wait 抽取同步 | FR-2.1.2-R1/R2/R3 |
| 3 | helper 无单测 | 写 `e2e/helpers/__tests__/env.test.ts` + `state.test.ts` | — | FR-2.2.4-R3（OQ-7） |
| 4 | smoke + ipc 耗时 ≥ 95s | wdio.conf 加 `restartStrategy: none` + `resetGoGuoState` | — | FR-2.2.1-R1/R2/R3 |
| 5 | 自启/复用双模式无区分 | wdio.conf 条件 + `helpers/env.ts` `shouldReuseDriver` | start/stop-driver.sh | FR-2.2.2-R1~R4 |
| 6 | wdio 日志有 plugin 警告 | Cargo.toml + lib.rs 注册 tauri-plugin-wdio | SC-8 全量回归 | FR-2.2.3-R1~R6 |
| 7 | 根 package.json 无 e2e 入口 | 加 `test:e2e` / `test:all` / `test:feature` / `test:e2e:feature` scripts + scripts/test-feature.mjs | — | FR-2.1.3-R1~R7 |
| 8 | 矩阵文档不存在 | 创建 `docs/test-level-matrix.md`（F201 9 行 + 等级原则） | 与 trace-matrix 双向链接 | FR-2.3.1-R1~R5 |
| 9 | AGENTS.md 无 §7 | 落盘 §7（含豁免清单）+ §4 交叉引用 + README 提示 | — | FR-2.4.3-R1~R5 |
| 10 | §N 模板文档不存在 | 创建 `docs/principles/test-design-section-template.md` | — | FR-2.4.4-R1~R3 |
| 11 | e2e README 无接入流程 | 加 Step 0 / 接入流程 / 已知限制三节 | — | FR-2.4.1-R1 / FR-2.5.2-R1/R3 |
| 12 | lint 脚本不存在 | 创建 `e2e/scripts/lint-specs.mjs` + `pnpm lint` 入口 | — | FR-2.4.2-R1 |
| 13 | setup-dev-env.sh 不存在 | 创建脚本 + 验证幂等 | — | FR-2.5.2-R2 |
| 14 | testing-principles.md 无 L1~L5 小节 | 加小节引用矩阵 | — | FR-2.3.2-R2 / FR-2.6.2-R1 |
| 15 | F114 PoC report §7.1 未标注 | 标注"已在 F115 实施" | — | FR-2.2.4-R2 / FR-2.6.2-R2 |
| 16 | 根 README 测试入口未含 e2e | 更新为 `pnpm test:all` | — | FR-2.6.2-R3 |
| 17 | ADR-0008 不存在 | 落盘（基于 §10 草稿） | — | FR-2.2.3-R6 |
| 18 | finalize 证据未产出 | 跑 97s benchmark + 10 连跑 flakiness + F201 演练 | workspace clean commit | SC-2 / SC-7 / C-P5 |

---

## 11. 不修改的内容

- F114 PoC 的核心断言（smoke + ipc 用例不改，仅迁移路径）
- e2e/.npmrc 镜像隔离策略（C-I4）
- release/data/mihomo/config.yaml 现有规则（C-I5）
- F109/F110/F114/F115/F101~F106 的 design.md / tasks.md（§7 豁免）
- F201 spec（C-P4：F115 不替 F201 写 spec/design）
- src-tauri/src/ 的业务逻辑（仅 lib.rs 注册 plugin 一行）

---

## 12. 验收证据与 finalize 检查清单

> 对齐 spec §9 验收证据清单，design 阶段补充"如何产出"。

| spec §9 项 | 产出方式 | 责任阶段 |
|-----------|---------|---------|
| e2e/specs/f114-baseline/ 迁移 | TDD 顺序 §N.5 #1 | M3 |
| helpers/{tauri-ipc,wait,env}.ts | TDD 顺序 §N.5 #2 | M3 |
| 根 package.json 四入口 | TDD 顺序 §N.5 #7 | M3 |
| `pnpm test:feature/e2e:feature -- f114-baseline` 跑通 | 同上验证 | M3 |
| 5 次连跑 ≤ 70s | `benchmark.sh` | M4 |
| wdio 日志无 plugin 警告 | `plugin-registered.spec.ts` | M4 |
| docs/test-level-matrix.md | TDD 顺序 §N.5 #8 | M5 |
| AGENTS.md §7 | TDD 顺序 §N.5 #9 | M5 |
| test-design-section-template.md | TDD 顺序 §N.5 #10 | M5 |
| e2e/README.md 三节 | TDD 顺序 §N.5 #11 | M5/M6 |
| setup-dev-env.sh | TDD 顺序 §N.5 #13 | M6 |
| F115 design.md §N（本节） | 已完成（design M1） | M1 |
| F201 演练通过 | F201 design 阶段（C-P4） | F201 |
| ADR-0008 落盘（决策 B：tasks 实测后一次性落盘最终版，design 阶段不创建文件） | TDD 顺序 §N.5 #17 | M4 |
| cargo test + pnpm test 全过 | SC-8 验证 | M4 |
| L1 单测覆盖 helpers | TDD 顺序 §N.5 #3 | M3 |
| 10 连跑 flakiness ≤ 10% | M8 finalize | M8 |
| PoC report §7.1 标注 | TDD 顺序 §N.5 #15 | M7 |
| testing-principles.md L1~L5 小节 | TDD 顺序 §N.5 #14 | M7 |
| 根 README test:all | TDD 顺序 §N.5 #16 | M7 |
| F110 §12 + GAP §9 已同步 | 已完成（spec v3 阶段） | ✅ |
| workspace clean commit | F115 finalize | M8（C-P5） |

---

## 13. design review 检查清单（建议 reviewer 关注）

- [ ] §2.3 tauri-plugin-wdio 进生产 Cargo.toml 决策是否需要再评估（vs dev-only gate）
- [ ] §4.3.3 F201 矩阵 `<TBD` 阈值修订建议是否接受（仅考核 L2 列 TBD ≤ 3.6）
- [ ] §5.1 AGENTS.md §7 豁免清单是否完整（是否漏掉某些已立项 Feature）
- [ ] §5.2 §N 模板"必填 vs 可选"清单是否合理
- [ ] §6.1 setup-dev-env.sh 是否覆盖所有首次配置场景（macOS/Windows 未来扩展）
- [ ] §N.5 TDD 顺序 18 步是否合理（是否漏掉关键步骤）
- [ ] §10 ADR-0008 决策是否需要补充量化数据（OQ-6 tasks 阶段实测后回填）
- [x] §10 ADR-0008 **落盘策略已确定**：决策 B — design 阶段不创建 docs/adr/0008 文件，tasks 阶段实测后于 §N.5 #17 一次性落盘最终版（2026-06-18）
