# F114 — UI E2E 自动化 PoC

- **类型**：基础设施 PoC（Spike）
- **状态**：✅ 已关闭（2026-06-18）
- **作者**：Teddy（项目管理者 & QA guardian）
- **Authority Source**：本报告（PoC 阶段，无 spec/design 走查）

## 1. 背景与动机

v0.1.0 共 625 测试，但全部停留在 **L1 unit / L2 FR acceptance / L3 contract+pipeline** 三层。
**L4（UI 行为）/ L5（桌面 E2E）完全缺失**，意味着：
- React 组件与 Rust Tauri 命令的"接线"是否在真实 webview 中工作，CI 无法验证
- 跨平台 GUI 回归（窗口冻结、IPC 时序、X11/Wayland 切换）只能人肉测
- F111 WSL2 休眠唤醒冻结这类问题，从测试侧无法提前发现

本 PoC 目标：**判断是否能在 WSL2 headless 下建立可持续的桌面 E2E 测试基础设施**，并产出三条可量化的 Quality Gate。

## 2. 决策记录

### 2.1 选型路径（已评审）

| 路径 | 方案 | 结论 |
|------|------|------|
| A | Playwright + Tauri webview | ✗ Playwright 不识别 wry webview |
| B | Tauri v2 内置 test API | ✗ 仅支持 Rust 侧 mock，无真实 webview |
| **C** | **tauri-driver + WebDriverIO + @wdio/tauri-service** | **✅ 官方正路，已验证** |
| D | CrabNebula 商业方案 | ✗ 需订阅，PoC 阶段不引入 |

**Path C 落地依据**：
- tauri-driver v2.0.6（crates.io）**未废弃**，是 Tauri v2 官方 WebDriver 桥
- WebKitWebDriver 2.52.3 来自 Ubuntu apt `webkit2gtk-driver`
- `@wdio/tauri-service` v1.1.0 是 WebDriverIO 官方维护的 Tauri 适配插件

### 2.2 误判修正

PoC 启动前的内部预判："tauri-driver 已废弃且不支持 Tauri v2" —— **错误**。
通过 web 检索 `v2.tauri.app/develop/tests/webdriver/` 与 `webdriver.io/docs/wdio-tauri-service/` 验证：tauri-driver 是 Tauri v2 在 Linux/Windows 的官方 WebDriver 实现，配合 `@wdio/tauri-service` 即开箱即用。

## 3. 落地工件

```
e2e/
├── README.md                    # 用户文档（含 QG 矩阵）
├── .npmrc                       # 阿里镜像隔离（避免污染主工程）
├── package.json                 # 独立 npm 包（与前端 vitest 解耦）
├── tsconfig.json
├── wdio.conf.ts                 # 配置：browserName=tauri, driverProvider=official
├── node_modules/                # gitignored
└── test/
    ├── specs/
    │   ├── smoke.spec.ts        # L4：窗口启动 + DOM 可达
    │   └── ipc.spec.ts          # QG2：add_target_site 真实 IPC roundtrip
    └── helpers/                 # 预留
```

### 3.1 关键配置点

| 配置 | 值 | 原因 |
|------|----|----|
| `browserName` | `"tauri"` | @wdio/tauri-service 注册名；非 `"wry"`（wdio v9 会拒绝） |
| `tauri:options.application` | `target/release/goguo` 绝对路径 | driver 通过此路径 spawn GoGuo |
| `services[0]` | `["@wdio/tauri-service", { driverProvider: "official" }]` | 复用 cargo install 装的 tauri-driver v2.0.6 |
| `GDK_BACKEND` | `x11` | **F111 教训强制项**，Wayland 在 WSL2 VM resume 后冻结 |
| 镜像 | npmmirror / rsproxy.cn | mihomo config 拦截 npmjs.org / crates.io（详见 §6） |

### 3.2 IPC 调用方式

```typescript
// 不依赖 UI 按钮，直接走 webview 内 __TAURI_INTERNALS__.invoke
await browser.executeAsync((cmd, args, done) => {
  window.__TAURI_INTERNALS__.invoke(cmd, args).then(done, e => done({ __error: String(e) }));
}, "add_target_site", { siteId: "github" });
```

理由：GoGuo 的 `withGlobalTauri=false`（默认），前端通过 ESM import 调 invoke；webview 内必须用 `__TAURI_INTERNALS__` 走最底层。

## 4. Quality Gate 验证证据

| QG | 标准 | 结果 | 证据 |
|----|------|------|------|
| QG1 | WSL2 headless 下能拉起 GoGuo 并访问 DOM | ✅ | browserName=wry, browserVersion=0.55.1, body element 定位成功 |
| QG2 | 一条真实 IPC roundtrip | ✅ | `add_target_site('github')` 返回 success=true, site.id='github' |
| QG3 | 连续 5 次运行 flakiness < 20% | ✅ | **5/5 PASS，flakiness=0%**，时长 91~103s，标准差小 |

QG3 失败教训：首轮用 90s timeout（实际需要 ~95s），全部"假阴性"。timeout 调到 180s 后零失败。

## 5. 风险登记

| 风险 | 等级 | 缓解 |
|------|------|------|
| **CI 环境跑不起 WebKitWebDriver** | HIGH | 需在 GitHub Actions Linux runner 装 webkit2gtk-driver；macOS 用 embedded provider |
| **tauri-driver 版本漂移** | MED | 锁定在 ~/.cargo + e2e/CI workflow 各自固定 |
| **首次启动 ~50s** | MED | 单 spec 跑 50s，双 spec 80s+。CI 可接受，本地开发建议 --spec 单跑 |
| **`@wdio/tauri-plugin` 未装** | LOW | 影响高级特性（mock、window 状态查询），不影响 QG1-3；正式 feature 阶段评估 |
| **负向 IPC 测试不可表达** | LOW | GoGuo 后端空 siteId 抛错，WebKitWebDriver 把 IPC 错误当 WebDriver 错误；负向行为应在 L2 用 Rust 测，不在 E2E 测 |

## 6. 环境发现（值得归档到 insights）

### 6.1 GoGuo 自身 mihomo 配置阻断开发工具链流量

**现象**：
- `cargo install tauri-driver` 直连 crates.io 失败：`SSL_ERROR_SYSCALL`
- `npm install` 直连 registry.npmjs.org 失败：同上
- 直连 IP（如 `curl -I https://registry.npmjs.org/`）超时

**根因**：`release/data/mihomo/config.yaml` 的规则：
- `site-crates` ruleset（line 98）和 `site-npmjs` ruleset（line 70）**存在但没有任何 DIRECT 规则匹配**
- `github.com` 在 line 2204 有 static IP + DIRECT，所以 GitHub 流量能过
- 但 `static.crates.io` / `registry.npmjs.org` 实际指向的 CDN IP 不在白名单

**绕过方案**（已在 PoC 应用）：
- `~/.cargo/config.toml` 加 `rsproxy-sparse` 镜像源
- `e2e/.npmrc` 加 npmmirror.com（隔离，不污染主工程）

**洞察**：GoGuo 的"自托管 mihomo 配置"在开发态是**双刃剑**——保护生产用户的同时，让开发态的 `cargo` / `pnpm` 无法工作。需要在正式 F-feature 中讨论是否：
- 在 mihomo config 加 `site-dev-tools` ruleset 让开发流量 DIRECT
- 或仅在开发环境用单独的 `config.dev.yaml`

### 6.2 capabilities 字段名坑

| 字段 | 错误值 | 正确值 |
|------|--------|--------|
| tauri:options | `binary` | **`application`** |
| browserName | `"wry"` | **`"tauri"`**（@wdio/tauri-service 注册） |

错误值会得到：MiniBrowser + 空 page，不报错但完全跑偏。

## 7. 后续 Action

### 7.1 立即（本周）
- [ ] 把 e2e/ 纳入 `.github/workflows/ci.yml`（Linux runner + webkit2gtk-driver）
- [ ] 写 macOS / Windows 平台验证（embedded / crabnebula provider）

### 7.2 正式 Feature 立项（建议 F115）
- [ ] spec：覆盖 6 个前端页面 × 主要交互路径（dashboard、site-rules、baseline 审查弹窗）
- [ ] design：L4 / L5 边界（哪些行为用 L4，哪些必须 L5）
- [ ] 评估是否引入 `@wdio/tauri-plugin`（生产代码侵入 vs 测试便利）
- [ ] visual regression：`@wdio/visual-service` 评估

### 7.3 工程改进（与正式 Feature 解耦）
- [ ] mihomo config 增加 dev tools 流量放行（或独立 dev config）
- [ ] 主 README.md 增加 `pnpm test:e2e` 入口

## 8. 关闭检查清单

- [x] QG1/QG2/QG3 全部通过
- [x] e2e/ 工件已落盘
- [x] 环境特殊配置（镜像、GDK_BACKEND）已记录
- [x] 后续 Action 已拆分到 F115 立项 + 立即项
- [x] 关键教训已写入 MEMORY.md（见 `patterns.md`）

## 9. 引用

- [Tauri v2 WebDriverIO 官方示例](https://v2.tauri.app/develop/tests/webdriver/example/webdriverio/)
- [@wdio/tauri-service 文档](https://webdriver.io/docs/wdio-tauri-service/)
- [tauri-driver crate](https://crates.io/crates/tauri-driver)
