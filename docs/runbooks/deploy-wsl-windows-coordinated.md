# GoGuo 部署 Runbook — Windows / WSL / Linux

- **适用版本**: v0.1.0+
- **更新日期**: 2026-05-27
- **mihomo 版本**: v1.19.25

---

## 1. 概述

GoGuo 是面向普通办公用户与 PC 端开发者的本地网络可达性工具，通过托管 mihomo 子进程实现代理规则管理。

### 部署模式

| 模式 | 说明 | 适用场景 |
|------|------|----------|
| `windows_only` | Windows 单平台 | 纯 Windows 办公用户 |
| `wsl_only` | WSL 单平台 | WSL 内独立使用 |
| `linux_only` | Linux 单平台 | Linux 桌面用户 |
| `coordinated` | Windows + WSL 协同 | Win/WSL 双侧同时管理 |

### 支持的协同方向

| 方向 | 运行位置 | 管理范围 | 桥接方式 |
|------|----------|----------|----------|
| A | Windows | Win + WSL | `wsl -e` 桥接 |
| B | WSL | Win + WSL | `powershell.exe` 桥接 |

---

## 2. 初始部署检查清单

> 首次部署按顺序执行以下步骤。每一步都有关键依赖，跳过会导致功能异常。

### 步骤 1：检查端口冲突

**部署前必须确认 7890 和 9090 端口未被占用。** 如果本机已有其他代理软件（如 GitHub-Host、Clash、V2Ray 等），必须先停止，否则 mihomo 无法启动或端口冲突导致流量走错代理。

```bash
# WSL / Linux
ss -tlnp | grep -E '7890|9090'

# Windows（PowerShell）
netstat -ano | findstr "7890"
```

**如果端口被占用**：

```powershell
# 查找并停止占用进程（Windows）
Get-Process -Id (Get-NetTCPConnection -LocalPort 7890).OwningProcess | Stop-Process -Force
```

```bash
# 查找并停止占用进程（Linux/WSL）
kill $(ss -tlnp | grep 7890 | grep -oP 'pid=\K\d+')
```

### 步骤 2：安装 mihomo 二进制

从 GoGuo Release 页面下载对应平台的 mihomo：

| 平台 | 文件名 |
|------|--------|
| Windows amd64 | `mihomo-windows-amd64-v1.19.25.zip` |
| Linux / WSL amd64 | `mihomo-linux-amd64-v1.19.25.gz` |

下载地址：https://github.com/slivelight/goguo/releases/tag/v0.1.0

**Windows**：

```powershell
mkdir "<DATA_DIR>\bin" -Force
Expand-Archive -Path mihomo-windows-amd64-v1.19.25.zip -DestinationPath <DATA_DIR>\bin
# 确认
Test-Path "<DATA_DIR>\bin\mihomo.exe"
```

**WSL / Linux**：

```bash
mkdir -p <DATA_DIR>/bin
gunzip -c mihomo-linux-amd64-v1.19.25.gz > <DATA_DIR>/bin/mihomo
chmod +x <DATA_DIR>/bin/mihomo
# 确认
ls -l <DATA_DIR>/bin/mihomo
```

### 步骤 3：配置 mihomo 代理节点

> **关键步骤。** mihomo 默认配置没有代理节点，GoGuo 启动后无法代理任何流量。必须手动配置代理节点（从订阅、节点服务商获取）。

编辑 `<DATA_DIR>/data/mihomo/config.yaml`，写入代理节点、代理组和规则。

**最小可用配置模板**：

```yaml
mixed-port: 7890
allow-lan: true          # coordinated 模式必须 true，允许跨平台访问
bind-address: "*"        # coordinated 方向 B 必须，绑定所有接口
mode: rule
log-level: warning
external-controller: 0.0.0.0:9090
secret: ""

dns:
  enable: true
  enhanced-mode: fake-ip
  fake-ip-range: 198.18.0.1/16
  nameserver:
    - 223.5.5.5
    - 119.29.29.29
  fallback:
    - https://dns.google/dns-query
    - https://cloudflare-dns.com/dns-query
  fallback-filter:
    geoip: true
    geoip-code: CN

proxies:
  # 在此添加代理节点，示例：
  - name: "my-proxy"
    type: ss
    server: <服务器地址>
    port: <端口>
    cipher: aes-256-gcm
    password: <密码>

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
      - "my-proxy"

rules:
  - GEOIP,CN,DIRECT
  - MATCH,PROXY
```

**从已有配置迁移**：如果从其他代理工具（如 GitHub-Host）迁移，可直接复制其 `proxies`、`proxy-groups`、`rules`、`rule-providers` 部分到 GoGuo 的 mihomo config.yaml。如有 ruleset 文件，需复制到 `<DATA_DIR>/data/mihomo/ruleset/` 目录并确保 `rule-providers` 路径正确。

**验证配置语法**：

```bash
python3 -c "import yaml; yaml.safe_load(open('<DATA_DIR>/data/mihomo/config.yaml')); print('OK')"
```

### 步骤 4：安装并启动 GoGuo

**Windows**：

```powershell
.\GoGuo_0.1.0_x64-setup.exe
```

**WSL / Linux（从源码构建）**：

```bash
cd <项目目录>
pnpm install && pnpm tauri build
sudo dpkg -i src-tauri/target/release/bundle/deb/goguo_0.1.0_amd64.deb
# 或直接运行 AppImage
```

启动后在向导中选择部署模式。

### 步骤 5：配置 WSL/Linux 代理环境变量

> GoGuo 启动后 mihomo 运行在本地 `127.0.0.1:7890`，但应用本身不会自动设置 shell 代理环境变量。需要手动配置，否则终端命令（curl、git 等）不走代理。

**当前会话立即生效**：

```bash
export http_proxy=http://127.0.0.1:7890
export https_proxy=http://127.0.0.1:7890
export no_proxy=localhost,127.0.0.1
```

**持久化（所有新终端自动生效）**：写入 `/etc/environment`（需要 sudo）：

```bash
sudo bash -c 'cat >> /etc/environment << EOF
http_proxy="http://127.0.0.1:7890"
HTTP_PROXY="http://127.0.0.1:7890"
https_proxy="http://127.0.0.1:7890"
HTTPS_PROXY="http://127.0.0.1:7890"
no_proxy="localhost,127.0.0.1"
NO_PROXY="localhost,127.0.0.1"
EOF'
```

> 注意：修改 `/etc/environment` 后需**重新打开终端**才生效。

**检查 `~/.profile` 或 `~/.bashrc` 中的代理覆盖**：

> **重要。** 如果之前使用过其他代理工具（如 GitHub-Host），可能在 `~/.profile` 中留下了自动探测网关 IP 并设置代理的脚本。这段脚本的执行优先级**高于** `/etc/environment`，会导致代理指向旧地址（如 Windows 侧网关 IP）。

检查是否存在旧代理配置：

```bash
grep -n -i proxy ~/.profile ~/.bashrc ~/.bash_profile 2>/dev/null
```

如果找到类似以下内容（探测网关 IP + 端口的自动代理脚本）：

```bash
# === Proxy config (auto-generated) ===
_proxy_host() { ip route show default 2>/dev/null | awk '{print $3}'; }
_ph=$(_proxy_host)
if [ -n "$_ph" ]; then
  for _port in 7890 7892; do
    if timeout 1 bash -c "echo >/dev/tcp/$_ph/$_port" 2>/dev/null; then
      export http_proxy="http://$_ph:$_port"
      ...
```

**必须替换**为指向本地 GoGuo mihomo 的固定配置：

```bash
# === Proxy config (GoGuo mihomo) ===
_proxy_port=7890
if timeout 1 bash -c "echo >/dev/tcp/127.0.0.1/$_proxy_port" 2>/dev/null; then
  export http_proxy="http://127.0.0.1:$_proxy_port"
  export https_proxy="http://127.0.0.1:$_proxy_port"
  export HTTP_PROXY="$http_proxy"
  export HTTPS_PROXY="$https_proxy"
  export no_proxy="localhost,127.0.0.1,::1"
  export NO_PROXY="$no_proxy"
fi
unset _proxy_port
# === End proxy config ===
```

修改后**重新打开终端**或 `source ~/.profile` 生效。

### 步骤 6：配置 Windows 系统代理（协同模式方向 B）

> 仅 WSL → 管理 Win + WSL 方向需要此步骤。GoGuo coordinated 模式通过 `WindowsRemoteAdapter` 自动完成，但首次部署时可能需要手动验证。

**获取 WSL IP**：

```bash
hostname -I | awk '{print $1}'
# 例如: 192.168.182.49
```

**设置 Windows IE 代理**：

```powershell
Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' -Name ProxyServer -Value '<WSL_IP>:7890'
# 确认
Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' | Select-Object ProxyEnable,ProxyServer
```

### 步骤 7：验证部署

**WSL / Linux**：

```bash
curl -s -I --max-time 5 -x http://127.0.0.1:7890 https://github.com | head -3
# 预期: HTTP/1.1 200 Connection established
```

**Windows（协同模式方向 B）**：

```powershell
# 验证 Windows 能连通 WSL mihomo
Test-NetConnection -ComputerName <WSL_IP> -Port 7890
# TcpTestSucceeded 应为 True

# 验证通过系统代理访问
$c = New-Object System.Net.WebClient
$c.DownloadString('https://github.com') | Select-Object Length
# 预期: 返回页面内容长度
```

**GoGuo UI 验证**：

1. 仪表盘显示正确的部署模式
2. 目标站点可达性状态为绿色
3. mihomo 进程运行中

---

## 3. 数据目录

GoGuo 使用 Tauri 标准 `app_data_dir`，路径因平台而异：

| 平台 | 路径 |
|------|------|
| Windows | `C:\Users\<用户>\AppData\Roaming\com.goguo.app\` |
| WSL / Linux | `~/.local/share/com.goguo.app/` |

> 以下简称 `<DATA_DIR>`。

### 完整目录结构

```text
<DATA_DIR>/
├── bin/
│   └── mihomo[.exe]              # mihomo 代理引擎
├── baseline/                     # 网络状态快照
├── config/
│   ├── settings.json             # 用户配置（含部署模式）
│   └── site-definitions/         # 目标站点定义
├── rules/
│   └── current-rules.yaml        # mihomo 规则
├── mihomo/
│   ├── config.yaml               # mihomo 运行配置（含代理节点）
│   ├── geoip.metadb              # GeoIP 数据库（mihomo 自带）
│   ├── geosite.dat               # GeoSite 数据库（mihomo 自带）
│   ├── cache.db                  # DNS 缓存
│   └── ruleset/                  # 自定义规则文件（可选）
│       ├── github.yaml
│       ├── github-ip.yaml
│       ├── custom-direct.yaml
│       ├── custom-proxy.yaml
│       └── custom-block.yaml
└── audit/
    └── audit-*.jsonl             # 操作审计日志
```

---

## 4. 协同模式专项

### 4.1 方向 A：Windows → 管理 Win + WSL

**前置条件**

| 条件 | 检查命令 | 说明 |
|------|----------|------|
| WSL2 已安装 | `wsl -l -v` | 版本 2 |
| Ubuntu/Debian 发行版 | `wsl -e cat /etc/os-release` | 已验证兼容 |
| curl（WSL 内） | `wsl -e curl --version` | 可达性探测 |

**配置步骤**

1. 在 Windows 启动 GoGuo
2. 向导中选择 `coordinated` 模式
3. GoGuo 自动检测 WSL 并创建 WslRemoteAdapter
4. mihomo 使用 Windows 版本（`bin\mihomo.exe`）

**桥接机制**：GoGuo 通过 `wsl -e <command>` 执行 WSL 侧操作（写入 `/etc/environment`、`/etc/resolv.conf` 等）。

### 4.2 方向 B：WSL → 管理 Win + WSL

**前置条件**

| 条件 | 检查命令 | 说明 |
|------|----------|------|
| PowerShell 可从 WSL 调用 | `powershell.exe -Command "echo ok"` | 桥接操作依赖 |

**配置步骤**

1. 在 WSL 启动 GoGuo
2. 向导中选择 `coordinated` 模式
3. GoGuo 自动检测 Windows 并创建 WindowsRemoteAdapter
4. mihomo 使用 Linux 版本（`bin/mihomo`）
5. mihomo 必须配置 `allow-lan: true` + `bind-address: "*"` + `external-controller: 0.0.0.0:9090`，允许 Windows 侧访问

**桥接机制**：GoGuo 通过 `powershell.exe -Command <cmd>` 执行 Windows 侧操作（设置系统代理、修改 hosts 等）。

### 4.3 WSL2 网络模式

GoGuo 自动检测 WSL2 网络模式并适配：

| 网络模式 | 方向 A（Win → WSL） | 方向 B（WSL → Win） |
|----------|---------------------|---------------------|
| NAT（默认） | 代理地址设为 WSL 网关 IP | mihomo 绑定 `0.0.0.0`；Windows 代理指向 WSL IP |
| Mirrored（镜像） | 共享 localhost | 共享 localhost |

检查当前模式：

```powershell
# Windows 中检查 .wslconfig
cat $env:USERPROFILE\.wslconfig
# networkingMode=mirrored → 镜像模式
```

### 4.4 适用场景

| 场景 | 推荐方向 | 原因 |
|------|----------|------|
| 开发者主力在 WSL | 方向 B | 开发工具链在 WSL，减少切换 |
| 办公用户主力在 Windows | 方向 A | 系统代理原生集成更好 |
| 需要系统级代理图标/通知 | 方向 A | Windows 系统托盘集成 |
| 需要轻量后台服务 | 方向 B | Linux 进程管理更轻量 |

---

## 5. 常见问题

### Q1: mihomo 进程无法启动

- **原因**: 二进制路径错误或权限不足
- **排查**:
  - 确认 `<DATA_DIR>/bin/mihomo` 存在
  - Linux/WSL: 确认有执行权限 `ls -l <DATA_DIR>/bin/mihomo`
  - Windows: 确认 `mihomo.exe` 未被杀毒软件拦截

### Q2: 端口 7890 被占用

- **原因**: 其他代理软件（GitHub-Host、Clash 等）仍在运行
- **排查**:
  ```bash
  # Linux/WSL
  ss -tlnp | grep 7890
  # Windows
  netstat -ano | findstr "7890"
  ```
- **解决**: 先停止其他代理软件，再启动 GoGuo

### Q3: WSL 代理环境变量不生效

- **原因**: `/etc/environment` 未写入代理配置，或修改后未重新打开终端
- **解决**:
  1. 确认 `/etc/environment` 包含 `http_proxy` 和 `https_proxy` 行
  2. 新开终端验证 `echo $http_proxy`
  3. GoGuo 会通过 F102 修复代码自动写入，但首次部署可能需要手动执行（需 sudo）

### Q4: Windows 浏览器无法走代理（协同模式方向 B）

- **原因**: IE 系统代理未指向 WSL IP
- **排查**:
  ```powershell
  Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' | Select-Object ProxyEnable,ProxyServer
  ```
- **解决**: ProxyServer 应为 `<WSL_IP>:7890`。参考步骤 6 设置

### Q5: WSL 未运行时无法配置（协同模式）

- **原因**: GoGuo 无法访问 WSL 进程
- **解决**: 先启动 WSL（Windows 终端运行 `wsl`），再操作 GoGuo

### Q6: 从其他代理工具迁移后流量异常

- **原因**: 旧代理进程未完全停止，或 mihomo config.yaml 中代理节点配置不正确
- **排查**:
  ```bash
  # 确认只有一个 mihomo 进程
  ps aux | grep mihomo
  # 确认流量走 GoGuo 的 mihomo
  curl -v -x http://127.0.0.1:7890 https://github.com 2>&1 | head -5
  ```
- **解决**: 停止旧代理进程，验证 config.yaml 中 `proxies` 部分节点可达

---

## 6. 回滚

### Windows

```powershell
# 1. 在 GoGuo UI 中点击"停止服务"
# 2. 清除 Windows 系统代理
Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' -Name ProxyEnable -Value 0
# 3. 卸载程序：控制面板 → 程序 → 卸载 GoGuo
# 4. 手动清理数据目录（可选）
Remove-Item -Recurse "C:\Users\<用户>\AppData\Roaming\com.goguo.app"
```

### WSL / Linux

```bash
# 1. 在 GoGuo UI 中点击"停止服务"
# 2. 清除 /etc/environment 中的代理行（需要 sudo）
sudo sed -i '/proxy\|PROXY/d' /etc/environment
# 3. 卸载（Deb 方式安装的）
sudo dpkg -r goguo
# 4. 手动清理数据目录（可选）
rm -rf ~/.local/share/com.goguo.app
```

---

## 7. 附录

### 7.1 数据目录权限

| 平台 | 说明 |
|------|------|
| Windows | 用户级权限（`%APPDATA%`） |
| WSL / Linux | `<DATA_DIR>` 用户级权限；`/etc/environment`、`/etc/resolv.conf` 写入需 sudo |
| 协同桥接 | 继承被调用侧的用户权限 |

### 7.2 端口说明

| 端口 | 用途 |
|------|------|
| 7890 | mihomo 混合代理端口（HTTP/SOCKS5） |
| 9090 | mihomo 外部控制器 API |

### 7.3 mihomo 热重载

修改 mihomo 配置后无需重启 GoGuo，可通过 API 热重载：

```bash
curl -X PUT http://127.0.0.1:9090/configs \
  -H "Content-Type: application/json" \
  -d '{"path":"<DATA_DIR>/data/mihomo/config.yaml"}'
```

### 7.4 配置文件参考

`<DATA_DIR>/config/settings.json` 示例：

```json
{
  "install_root": "<DATA_DIR>",
  "deployment_mode": "coordinated",
  "mihomo": {
    "binary_path": "<DATA_DIR>/bin/mihomo",
    "config_dir": "<DATA_DIR>/data/mihomo",
    "api_address": "127.0.0.1:9090",
    "mixed_port": 7890
  },
  "proxy_guard": {
    "check_interval_secs": 3,
    "max_restart_attempts": 3
  }
}
```
