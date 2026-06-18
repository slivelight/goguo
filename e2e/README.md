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

```bash
# WSL2 必须强制 X11（F111 教训）
GDK_BACKEND=x11 pnpm test

# 指定不同的 GoGuo 二进制
GOGUO_BIN=/path/to/goguo pnpm test
```

## Quality Gates（PoC 阶段，2026-06-18 验证）

| QG | 标准 | 结果 | 证据 |
|----|------|------|------|
| QG1 | WSL2 headless 下能拉起 GoGuo 并访问 DOM | ✅ 通过 | browserName=wry, browserVersion=0.55.1, body 定位成功 |
| QG2 | 一条真实 IPC roundtrip（add_target_site） | ✅ 通过 | `add_target_site('github')` 返回 success=true, site.id='github' |
| QG3 | 连续 5 次运行 flakiness < 20% | ✅ 通过 | **5/5 PASS，flakiness=0%**，时长 91~103s |

完整 PoC 报告：`features/114-ui-e2e-poc/poc-report.md`
