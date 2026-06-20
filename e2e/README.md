# GoGuo E2E 测试

基于 **WebDriverIO v9 + tauri-driver** 的桌面 E2E 测试套件。

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

### 复用模式（日常开发，快 ~30s）

外部预启 tauri-driver 常驻，wdio 跳过 @wdio/tauri-service 直连。每次跑 ~36s（省 ~30s，FR-2.2.2-R3 要求 ≥8s，**3.9x 超额**）。

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
