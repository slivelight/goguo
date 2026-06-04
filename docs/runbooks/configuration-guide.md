# GoGuo 配置指南

- **适用版本**: v0.1.0+
- **更新日期**: 2026-06-02
- **目标读者**: GoGuo 用户（首次安装后配置）

---

## 1. 概述

GoGuo 安装后，部分配置由 GoGuo 自动管理，部分需要用户手动准备。本文档明确区分两者，并提供手动配置的操作指导。

### 数据目录

GoGuo 采用便携包分发，**安装根目录（`INSTALL_ROOT`）即用户解压发布包的目标目录**，所有数据和配置存放在 `<INSTALL_ROOT>/data/` 下：

| 模式 | INSTALL_ROOT | 数据目录 |
|------|-------------|---------|
| 生产模式 | 可执行文件所在目录（用户解压位置） | `<INSTALL_ROOT>/data/` |
| 开发模式（`pnpm tauri dev`） | `<项目根目录>/release/` | `<release>/data/` |

> 以下简称 `<INSTALL_ROOT>` 为安装根目录，数据目录为 `<INSTALL_ROOT>/data/`。

---

## 2. 配置文件分类总览

### 2.1 GoGuo 自动管理（用户无需手动干预）

| 文件 | 说明 | 自动管理方式 |
|------|------|-------------|
| `<INSTALL_ROOT>/data/config/settings.json` | 应用主配置 | 首次启动时自动生成默认值；Wizard 选择部署模式时自动更新 |
| `<INSTALL_ROOT>/data/rules/current-rules.yaml` | 当前生效的代理规则 | 添加/移除站点时自动生成并写入 |
| `<INSTALL_ROOT>/data/rules/previous-rules.yaml` | 规则备份 | 更新规则前自动备份，用于回滚 |
| `<INSTALL_ROOT>/data/mihomo/config.yaml` | mihomo 运行配置 | GoGuo 通过 MihomoManager 写入启动配置 |
| `<INSTALL_ROOT>/data/baseline/initial-snapshot.json` | 初始网络状态快照 | Wizard "初始评估" 步骤触发采集 |
| `<INSTALL_ROOT>/data/baseline/baseline-v1.json` | 已确认的 baseline | Wizard "确认 Baseline" 步骤触发保存 |
| `<INSTALL_ROOT>/data/audit/*.jsonl` | 操作审计日志 | 所有操作自动追加记录 |

### 2.2 GoGuo 自动管理但需前置条件

| 文件/系统 | 说明 | 前置条件 |
|----------|------|---------|
| `/etc/environment` | WSL/Linux 代理环境变量 | 需要 root/sudo 权限写入代理行；停止服务时自动清除 |
| `/etc/resolv.conf` | DNS 配置恢复 | 需要 root/sudo 权限；仅恢复 baseline 中记录的原始值 |
| Windows 注册表 (IE 代理) | Windows 系统代理设置 | 协同模式下 GoGuo 通过 PowerShell 自动管理 |
| `~/.gitconfig` | git 代理设置 | GoGuo 通过 `git config --global` 管理，需 git 可用 |

### 2.3 需用户手动配置（核心必做项）

| 文件 | 说明 | 必要性 |
|------|------|--------|
| `<INSTALL_ROOT>/bin/mihomo[.exe]` | mihomo 代理引擎二进制 | **必须** — GoGuo 依赖 mihomo 提供代理服务 |
| `<INSTALL_ROOT>/data/mihomo/config.yaml` 中的 `proxies` 段 | 代理节点配置 | **必须** — 没有代理节点，GoGuo 无法代理任何流量 |
| `<INSTALL_ROOT>/data/mihomo/config.yaml` 中的 `proxy-groups` 段的 `proxies` 列表 | 代理组引用的节点 | **必须** — 需与 proxies 段中的节点名称一致 |

### 2.4 需用户手动配置（可选增强项）

| 文件 | 说明 | 必要性 |
|------|------|--------|
| `<INSTALL_ROOT>/data/mihomo/ruleset/custom-proxy.yaml` | 用户自定义代理规则 | 可选 — 添加走代理的域名规则 |
| `<INSTALL_ROOT>/data/mihomo/ruleset/custom-direct.yaml` | 用户自定义直连规则 | 可选 — 添加走直连的域名规则 |
| `<INSTALL_ROOT>/data/mihomo/ruleset/custom-block.yaml` | 用户自定义拦截规则 | 可选 — 添加被拦截的域名规则 |
| `%USERPROFILE%\.wslconfig` | WSL 网络模式配置 | 可选 — 协同模式下切换 mirrored 网络可简化配置 |

---

## 3. 必做手动配置操作

### 3.1 安装 mihomo 二进制

> **前置条件**：GoGuo 无 mihomo 二进制则无法启动代理服务。

从 GoGuo Release 页面下载对应平台的 mihomo：

| 平台 | 文件名 | 解压目标路径 |
|------|--------|------------|
| Windows amd64 | `mihomo-windows-amd64-v1.19.25.zip` | `<INSTALL_ROOT>\bin\mihomo.exe` |
| Linux / WSL amd64 | `mihomo-linux-amd64-v1.19.25.gz` | `<INSTALL_ROOT>/bin/mihomo` |

下载地址：https://github.com/slivelight/goguo/releases/tag/v0.1.0

**Windows**：

```powershell
mkdir "<INSTALL_ROOT>\bin" -Force
Expand-Archive -Path mihomo-windows-amd64-v1.19.25.zip -DestinationPath <INSTALL_ROOT>\bin
Test-Path "<INSTALL_ROOT>\bin\mihomo.exe"
```

**WSL / Linux**：

```bash
mkdir -p <INSTALL_ROOT>/bin
gunzip -c mihomo-linux-amd64-v1.19.25.gz > <INSTALL_ROOT>/bin/mihomo
chmod +x <INSTALL_ROOT>/bin/mihomo
ls -l <INSTALL_ROOT>/bin/mihomo
```

### 3.2 配置 mihomo 代理节点

> **核心步骤**。mihomo 默认配置没有代理节点，GoGuo 启动后无法代理任何流量。

编辑 `<INSTALL_ROOT>/data/mihomo/config.yaml`，在 `proxies:` 段下添加代理节点。

**代理节点来源**：
- 从订阅服务获取（节点服务商、机场订阅链接）
- 手动填写（从服务商提供的节点信息）

**常见节点类型示例**：

```yaml
proxies:
  # Shadowsocks 节点
  - name: "my-ss-node"
    type: ss
    server: <服务器地址>
    port: <端口>
    cipher: aes-256-gcm
    password: <密码>

  # VMess 节点
  - name: "my-vmess-node"
    type: vmess
    server: <服务器地址>
    port: <端口>
    uuid: <UUID>
    alterId: 0
    cipher: auto
    network: ws
    ws-opts:
      path: /
      headers:
        Host: <服务器地址>

  # Trojan 节点
  - name: "my-trojan-node"
    type: trojan
    server: <服务器地址>
    port: 443
    password: <密码>
    sni: <SNI域名>
```

**从已有配置迁移**：如果从其他代理工具迁移，可直接复制其 `proxies` 部分到 GoGuo 的 mihomo config.yaml。

### 3.3 配置 mihomo 代理组

在 `<INSTALL_ROOT>/data/mihomo/config.yaml` 的 `proxy-groups:` 段中，将代理节点名称加入对应组的 `proxies:` 列表：

```yaml
proxy-groups:
  - name: PROXY
    type: select
    proxies:
      - GitHub
      - DIRECT

  - name: GitHub
    type: url-test
    tolerance: 100
    interval: 300
    lazy: true
    url: http://www.gstatic.com/generate_204
    proxies:
      - "my-ss-node"        # ← 替换为你添加的节点名称
      - "my-vmess-node"     # ← 节点名称必须与 proxies 段中的 name 一致
```

> **关键**：`proxy-groups` 中引用的节点名称必须与 `proxies` 段中的 `name` 字段完全一致。

---

## 4. 可选手动配置操作

### 4.1 自定义代理规则

mihomo 规则分为三类，用户可通过编辑对应 YAML 文件添加自定义域名规则：

**4.1.1 自定义直连规则** — `<INSTALL_ROOT>/data/mihomo/ruleset/custom-direct.yaml`

国内域名或不需要走代理的域名：

```yaml
payload:
  - DOMAIN-SUFFIX,my-company.com
  - DOMAIN-SUFFIX,local-service.cn
```

**4.1.2 自定义代理规则** — `<INSTALL_ROOT>/data/mihomo/ruleset/custom-proxy.yaml`

需要走代理但不在 github.yaml 中的域名：

```yaml
payload:
  - DOMAIN-SUFFIX,models.dev
  - DOMAIN-SUFFIX,my-cloud-service.com
```

**4.1.3 自定义拦截规则** — `<INSTALL_ROOT>/data/mihomo/ruleset/custom-block.yaml`

需要完全拦截的域名：

```yaml
payload:
  - DOMAIN-SUFFIX,ads-tracker.com
```

修改规则文件后，通过 mihomo API 热重载：

```bash
curl -X PUT http://127.0.0.1:9090/configs \
  -H "Content-Type: application/json" \
  -d '{"path":"<INSTALL_ROOT>/data/mihomo/config.yaml"}'
```

或在 GoGuo UI 中操作站点管理，规则变更会自动触发重载。

### 4.2 WSL 网络模式切换

> 仅协同模式需要关注。

WSL2 默认 NAT 网络模式，Windows 和 WSL IP 不同。切换到 mirrored 模式可让两侧共享 localhost，简化配置。

编辑 `%USERPROFILE%\.wslconfig`（Windows 侧）：

```ini
[wsl2]
networkingMode=mirrored
```

修改后需重启 WSL：`wsl --shutdown`，再重新打开 WSL终端。

GoGuo 会自动检测当前网络模式并适配代理配置（NAT 模式指向 WSL IP，mirrored 模式指向 localhost）。

### 4.3 DNS 配置调整

mihomo config.yaml 中的 DNS 配置已预设为推荐值：

```yaml
dns:
  enable: true
  enhanced-mode: fake-ip        # fake-ip 模式：代理域名不需要本地DNS解析
  fake-ip-range: 198.18.0.1/16
  nameserver:
    - 223.5.5.5                 # 阿里DNS（国内域名）
    - 119.29.29.29              # 腾讯DNS（国内域名）
  fallback:
    - https://dns.alidns.com/dns-query   # 阿里DoH（国外域名，国内可达）
    - https://doh.pub/dns-query           # 腾讯DoH（国外域名，国内可达）
  fallback-filter:
    geoip: true
    geoip-code: CN
```

> **不建议修改** 此 DNS 配置，除非你有明确的网络需求。当前配置已解决国内环境下的 DNS 循环依赖问题。

---

## 5. 规则链与流量走向

理解规则链有助于判断哪些流量走代理、哪些走直连：

```
请求进入 mihomo
  → custom-direct.yaml 命中? → DIRECT（直连）
  → custom-block.yaml 命中? → REJECT（拦截）
  → custom-proxy.yaml 命中? → GitHub 代理组（走代理）
  → github.yaml 命中?       → GitHub 代理组（走代理）
  → github-ip.yaml 命中?    → GitHub 代理组（走代理）
  → GeoIP 判断为 CN?        → DIRECT（国内IP直连）
  → 未匹配任何规则?          → PROXY 代理组（走代理）
```

> 最终的 `MATCH,PROXY` 规则确保未匹配的域名默认走代理而非直连。这在 fake-ip 模式下不增加本地 DNS 压力——代理域名由代理服务器侧完成解析。

---

## 6. 完整目录结构参考

```text
<INSTALL_ROOT>/                              # 安装根目录（用户解压位置）
├── goguo.exe / gogugo.AppImage              # [手动] GoGuo 主程序
├── bin/
│   └── mihomo[.exe]                         # [手动] mihomo 代理引擎二进制
├── data/                                    # 所有运行时数据
│   ├── config/
│   │   ├── settings.json                    # [自动] GoGuo 应用配置
│   │   ├── subscription-sources.json        # [自动] 订阅源配置
│   │   └── site-definitions/                # [自动] GoGuo 站点定义
│   │       ├── github.json
│   │       ├── npmjs.json
│   │       └── custom/
│   ├── baseline/
│   │   ├── initial-snapshot.json            # [半自动] 需 Wizard 触发
│   │   └── baseline-v1.json                 # [半自动] 需 Wizard 确认
│   ├── rules/
│   │   ├── current-rules.yaml               # [自动] 当前生效规则
│   │   └── previous-rules.yaml              # [自动] 规则备份
│   ├── mihomo/
│   │   ├── config.yaml                      # [混合] DNS/规则等由 GoGuo 管理，proxies/proxy-groups 需手动
│   │   ├── geoip.metadb                     # [自动] mihomo 自带的 GeoIP 数据库
│   │   ├── geosite.dat                      # [自动] mihomo 自带的 GeoSite 数据库
│   │   ├── cache.db                         # [自动] DNS 缓存，运行时生成
│   │   └── ruleset/
│   │       ├── github.yaml                  # [自动] GitHub/AI 域名规则
│   │       ├── github-ip.yaml               # [自动] GitHub IP 段规则
│   │       ├── custom-direct.yaml            # [手动可选] 用户自定义直连规则
│   │       ├── custom-proxy.yaml             # [手动可选] 用户自定义代理规则
│   │       └── custom-block.yaml             # [手动可选] 用户自定义拦截规则
│   └── audit/
│       └─ audit-*.jsonl                     # [自动] 操作审计日志
└── storage/                                  # [自动] Tauri webview 存储
```

---

## 7. 配置验证清单

安装完成后，按以下顺序验证：

- [ ] mihomo 二进制存在且有执行权限
  ```bash
  # Linux/WSL
  ls -l <INSTALL_ROOT>/bin/mihomo
  # Windows
  Test-Path <INSTALL_ROOT>\bin\mihomo.exe
  ```

- [ ] mihomo config.yaml 包含至少 1 个代理节点
  ```bash
  grep "^  - name:" <INSTALL_ROOT>/data/mihomo/config.yaml | head -3
  ```

- [ ] proxy-groups 中引用了代理节点名称
  ```bash
  grep "proxies:" <INSTALL_ROOT>/data/mihomo/config.yaml
  ```

- [ ] GoGuo Wizard 完成 7 步流程（部署模式 → 评估 → 确认 → 站点选择 → 预览 → 完成）

- [ ] mihomo 进程运行正常
  ```bash
  curl -s http://127.0.0.1:9090/version
  # 预期: {"meta":true,"version":"v1.19.25"}
  ```

- [ ] 代理功能正常
  ```bash
  curl -s -I --max-time 5 -x http://127.0.0.1:7890 https://github.com | head -2
  # 预期: HTTP/1.1 200 Connection established
  ```

---

## 8. 常见配置问题

### Q1: GoGuo 启动后 mihomo 报错 "binary not found"

- **原因**: `<INSTALL_ROOT>/bin/mihomo` 文件不存在
- **解决**: 按步骤 3.1 安装 mihomo 二进制

### Q2: mihomo 启动后所有流量走直连，国外网站无法访问

- **原因**: `proxies` 段为空或代理节点不可用
- **解决**: 按步骤 3.2 配置代理节点；确保节点服务器可达

### Q3: DNS 解析失败（日志中出现 "dns resolve failed"）

- **原因**: fallback DNS 不可达（如使用了 cloudflare-dns.com 等被墙的 DoH）
- **解决**: 确认 config.yaml 中 fallback 使用国内可达的 DoH（`dns.alidns.com` / `doh.pub`）

### Q4: 规则修改后不生效

- **原因**: mihomo 配置未热重载
- **解决**: 通过 API 重载或在 GoGuo UI 中操作站点管理触发重载

### Q5: WSL 协同模式下 Windows 浏览器无法走代理

- **原因**: Windows 系统代理未指向 WSL 的 mihomo 地址
- **解决**: GoGuo 协同模式 B 会自动配置；如未生效，检查 Windows 注册表代理设置或 WSL 网络模式