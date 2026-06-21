# GoGuo E2E 测试

基于 **WebDriverIO v9 + tauri-driver** 的桌面 E2E 测试套件。

## Step 0: 开发环境首次配置（必读）

> F115 FR-2.5.2-R1 引入。处理 **GAP-F115-2**（mihomo config 阻断 cargo/pnpm 流量）的开发态缓解；根因修复推到 F116+（详见 `features/110-design-gap-closure/design.md` §12 GAP-F115-2）。

**一键配置**（推荐，FR-2.5.2-R2，T-19 已实施）：

```bash
cd e2e && bash scripts/setup-dev-env.sh
```

脚本三步骤（幂等，重复执行不重复写入）：

1. **cargo 镜像配置**：检测 `~/.cargo/config.toml` 是否含 rsproxy 配置（宽容识别 `rsproxy` / `rsproxy-sparse` / `replace-with` 变体），若无则追加
2. **e2e/.npmrc 校验**：C-I4 隔离策略必需，文件必须存在且含 npmmirror 配置
3. **cargo 网络可达验证**：`cargo install tauri-driver --dry-run` 触发镜像索引 fetch，检测 SSL_ERROR / network 错误

**平台适用性**（id:05 周边）：

| 平台 | 行为 |
|------|------|
| WSL2 / Linux | 执行三步骤（mihomo 阻断影响）|
| macOS / Windows | SKIP exit 0（直连 crates.io / npmjs.org 可达）|

**手动配置**（当脚本不可用或需定制时）：

- `~/.cargo/config.toml` 加 `rsproxy-sparse` 镜像源（处理 cargo 流量阻断）：

  ```toml
  [source.rsproxy-sparse]
  registry = "sparse+https://rsproxy.cn/index/"

  [source.crates-io]
  registry = "sparse+https://index.crates.io/"
  replace-with = "rsproxy-sparse"
  ```

- `e2e/.npmrc` 含 npmmirror.com（C-I4 隔离策略，处理 pnpm 流量阻断）：

  ```ini
  registry=https://registry.npmmirror.com/
  ```

**校验**：`cargo install tauri-driver --dry-run` + `pnpm install --dry-run` 不报 SSL 错误。

## 架构

```
wdio (Node) ──W3C WebDriver──▶ tauri-driver:4444 ──▶ WebKitWebDriver:4445 ──▶ GoGuo (release binary)
```

- tauri-driver：Tauri 官方 WebDriver 桥（[crates.io](https://crates.io/crates/tauri-driver) v2.0.6）
- WebKitWebDriver：来自 apt 包 `webkit2gtk-driver`（Linux）
- GoGuo 二进制：`target/release/goguo`（由 wdio 通过 `tauri:options.application` 拉起）

## 准备

1. **构建 GoGuo release**（仓库根）：
   ```bash
   cd src-tauri && cargo build --release
   ```

2. **安装系统依赖**（一次性）：
   ```bash
   sudo apt install -y webkit2gtk-driver
   cargo install tauri-driver
   ```

3. **安装 npm 依赖**（本目录）：
   ```bash
   cd e2e && pnpm install   # 或 npm install
   ```

## 运行

### 自启模式（默认，首次/CI 验证）

@wdio/tauri-service 全权管理 tauri-driver 生命周期——每次跑前后自动 spawn/kill。隔离性强，但每次跑 ~1m6s。

```bash
# WSL2 自动强制 X11（onPrepare 内 ensureX11Backend，F111 教训）
pnpm test

# 指定不同的 GoGuo 二进制
GOGUO_BIN=/path/to/goguo pnpm test
```

### 复用模式（dev 体验可选项）

外部预启 tauri-driver 常驻，wdio 跳过 @wdio/tauri-service 直连。

> **⚠️ 勘误-4 同步（2026-06-20）**：T-12 度量发现 **post-T-09（plugin 接入后）复用模式不再快于自启模式**——5 次均值复用 43.82s（stddev 14.90s，前 3 次稳定 33~34s，run 4/5 退化到 52s/66s）vs 自启 28.95s（stddev 2.40s）。复用模式现定位为 **dev 体验可选项**（避免每次 spawn driver 的固定开销 + 便于迭代调试），SC-2 ≤70s 核心达标由自启模式承担。复用模式 stddev 不稳定问题挂账 F116+ 排查。详见 `features/115-ux-e2e-infrastructure/spec.md` 勘误-4 + `evidence/benchmark-M4.md`。

```bash
# 1. 预启 tauri-driver（幂等：已监听则直接 exit 0）
./scripts/start-driver.sh

# 2. 跑测试（可连跑多次）
TAURI_DRIVER_REUSE=1 pnpm test
TAURI_DRIVER_REUSE=1 pnpm test

# 3. 用完关闭
./scripts/stop-driver.sh
```

可选 env（默认值满足绝大多数场景）：

| env | 默认 | 说明 |
|-----|------|------|
| `TAURI_DRIVER_REUSE` | 未设 | `1` 启用复用模式，其它值/未设走自启模式 |
| `TAURI_DRIVER_PORT` | `4444` | 复用模式 tauri-driver 端口 |
| `TAURI_DRIVER_NATIVE_PORT` | `4445` | 复用模式 WebKitWebDriver 端口 |
| `TAURI_DRIVER_PID_FILE` | `/tmp/tauri-driver.pid` | pid 文件路径 |

> **dev 模式约束（spec FR-2.2.2-R1）**：tauri-driver 是 e2e 测试专用工具。生产模式（`<install-dir>/goguo`，面向最终用户）不启动 tauri-driver、不跑 wdio。

## Quality Gates（PoC 阶段，2026-06-18 验证）

| QG | 标准 | 结果 | 证据 |
|----|------|------|------|
| QG1 | WSL2 headless 下能拉起 GoGuo 并访问 DOM | ✅ 通过 | browserName=wry, browserVersion=0.55.1, body 定位成功 |
| QG2 | 一条真实 IPC roundtrip（add_target_site） | ✅ 通过 | `add_target_site('github')` 返回 success=true, site.id='github' |
| QG3 | 连续 5 次运行 flakiness < 20% | ✅ 通过 | **5/5 PASS，flakiness=0%**，时长 91~103s |

完整 PoC 报告：`features/114-ui-e2e-poc/poc-report.md`

## 单 Feature 测试入口（F115 FR-2.1.3-R4~R7）

### 用法

```bash
# 三层全跑（cargo 后端 + vitest 前端 + e2e）
pnpm test:feature -- <feature-id>

# 仅 e2e 层
pnpm test:e2e:feature -- <feature-id>
```

### feature-id 命名约定

格式：`f<NNN>` 或 `f<NNN>-<slug>`（与 `e2e/specs/` 子目录、`features/` 子目录同名）。

- **短形式 `f<NNN>`**：用于 cargo / vitest 过滤（匹配模块/文件名前缀）
- **长形式 `f<NNN>-<slug>`**：用于 e2e 层定位 `e2e/specs/<feature-id>/` 目录

### 可用 feature-id 清单（R7 自动补全降级方案）

> pnpm 不支持 `--<Tab>` 自动补全，以下清单手动维护。新增 feature spec 后请同步本表。

| feature-id | e2e spec | 后端 cargo 模块 | 前端 vitest 文件 |
|------------|---------|----------------|-----------------|
| `f114-baseline` | ✅ `specs/f114-baseline/`（smoke + ipc） | — | — |
| `f201-first-run` | ⏳ 待接入（首个正式案例） | — | — |

发现可用 feature-id 的快捷方式：

```bash
ls e2e/specs/                            # 现有 e2e spec 子目录
ls features/ | grep -E '^[0-9]'          # 现有 feature 目录
```

### 三层职责与降级

| 层 | 命令 | 无测试时行为 |
|---|---|---|
| L1+L2+L3 后端 | `cargo test --workspace -- f<NNN>` | PASS（cargo 自然行为，0 tests run = exit 0） |
| L4 前端 | `vitest run src/__tests__/**/*f<NNN>* --passWithNoTests` | PASS（`--passWithNoTests` 让"无匹配"= exit 0） |
| L5 e2e | `cd e2e && pnpm test:e2e:feature -- <id>` | SKIP（`specs/<id>/` 不存在时优雅跳过） |

## Feature 接入流程（FR-2.4.1-R1）

新 Feature 接入 e2e 测试基础设施的 5 步：

| Step | 动作 | 验证 |
|------|------|------|
| 1 | 在 `e2e/specs/f<NNN>-<slug>/` 创建目录 | `ls e2e/specs/` 看到新目录 |
| 2 | 复用 `e2e/helpers/` 中的 helper，不 inline `__TAURI_INTERNALS__` | `pnpm lint`（FR-2.4.2-R1）全过 |
| 3 | 在 `docs/test-level-matrix.md` 为本 Feature 每个 FR 添加 L1~L5 责任行 | 矩阵新行等级标注完整 |
| 4 | spec 命名约定：`<scenario>-<action>.spec.ts`（如 `dashboard-eval-trigger.spec.ts`） | 文件名符合 |
| 5 | 每个 spec 以 `describe("<Feature ID>: <scenario>")` 开头 | `pnpm lint` 校验通过 |

**配套要求**（来自 AGENTS.md §7）：
- `hf-design` 阶段必须在 design.md 中填写 `§N L1~L5 自动化测试设计` 章节（模板：`docs/principles/test-design-section-template.md`）
- 章节未通过 review 不得进入 `hf-tasks`
- F201 为首个按规范接入的 Feature 案例（FR-2.4.1-R2）

## 已知限制（FR-2.5.2-R3）

> F115 引入。多实例共存场景下的已知问题，**不在 F115 范围内修复**。详见 `features/110-design-gap-closure/design.md` §12 + GAP 索引 §9。

| 限制 | 严重度 | 描述 | 移交 |
|------|-------|------|------|
| `/etc/environment` 多实例覆盖 | 🔴 HIGH | `~/apps/goguo`（生产）与 `target/release/goguo`（dev）同时运行时，`write_state` 互相覆盖 `/etc/environment` | F110 §12 GAP-F115-1（建议 F116+） |
| mihomo config 阻断 cargo/pnpm | 🟡 MED | `site-crates` / `site-npmjs` ruleset 无 DIRECT 规则，开发态依赖 Step 0 镜像绕过 | F110 §12 GAP-F115-2 |
| mihomo config dev/prod 拆分 | 🟢 LOW | 单一 config 文件耦合开发态与生产态规则，dev 改动可能影响 prod 行为 | F110 §12 GAP-F115-3 |

> 当前缓解仅覆盖**开发态**镜像绕过（Step 0），**不影响**生产用户行为（spec C-I5 保持）。
