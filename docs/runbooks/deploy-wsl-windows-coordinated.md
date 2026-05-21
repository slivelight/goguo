# GoGuo 协同部署 Runbook — Windows + WSL 双向

- **部署模式**: Coordinated（协同模式）
- **适用版本**: v0.1.0+
- **日期**: 2026-05-21
- **目标**: 单实例运行 GoGuo，自动管理 Windows + WSL **两侧**网络配置，修改后双侧同步生效
- **支持的部署方向**:
  - **方向 A**: Windows 单实例 → 管理 Win + WSL
  - **方向 B**: WSL 单实例 → 管理 Win + WSL

---

## 方向 A：Windows 单实例 → 管理 Win + WSL

在 Windows 上运行 GoGuo，通过 WslRemoteAdapter（`wsl -e` 桥接）管理 WSL 侧配置。

### A1. 前置条件

| 条件 | 检查命令 | 说明 |
|------|----------|------|
| Node.js 18+ | `node --version` | LTS 版本 |
| pnpm | `pnpm --version` | 包管理器 |
| Rust stable | `rustc --version` | Tauri 后端编译 |
| Visual Studio Build Tools | — | C++ 工具链（Tauri 依赖） |
| WebView2 | — | Windows 10/11 已内置 |

**WSL 侧**

| 条件 | 检查命令 | 说明 |
|------|----------|------|
| WSL2 已安装 | `wsl -l -v`（Windows） | 版本 2 |
| Ubuntu/Debian 发行版 | `cat /etc/os-release`（WSL） | 已验证兼容 |
| curl | `curl --version`（WSL） | 可达性探测 |
| git | `git --version`（WSL） | 代理配置项 |

**mihomo 二进制**

- 版本: mihomo v1.18.x（MetaKernel 分支）
- 平台: Windows amd64
- 下载: https://github.com/MetaCubeX/mihomo/releases

### A2. 构建步骤（Windows PowerShell）

项目源码在 WSL `/home/slivelight/projects/goguo`，需在 Windows 访问：

```powershell
# 进入项目目录（通过 WSL 路径访问）
cd \\wsl$\Ubuntu\home\slivelight\projects\goguo

# 或映射到 Windows 路径
# 假设 WSL 挂载在 \\wsl.localhost\Ubuntu
cd \\wsl.localhost\Ubuntu\home\slivelight\projects\goguo
```

**安装依赖**

```powershell
# 前端依赖
pnpm install

# Rust 依赖（自动拉取）
cargo fetch
```

**构建 Windows 安装包**

```powershell
# 完整构建（生成 .msi / .exe 安装包）
pnpm tauri build

# 输出位置
# src-tauri\target\release\bundle\msi\GoGuo_0.1.0_x64.msi
# src-tauri\target\release\bundle\nsis\GoGuo_0.1.0_x64-setup.exe
```

### A3. 安装与配置

```powershell
# 运行安装程序
.\src-tauri\target\release\bundle\nsis\GoGuo_0.1.0_x64-setup.exe

# 默认安装位置
# C:\Users\<用户>\AppData\Local\Programs\GoGuo\
```

**准备 mihomo 二进制**

```powershell
# 创建 bin 目录
mkdir "C:\Users\<用户>\AppData\Local\Programs\GoGuo\bin"

# 下载 mihomo Windows 版本
# 从 https://github.com/MetaCubeX/mihomo/releases 下载
# mihomo-windows-amd64-v1.18.x.zip

# 解压并放置
# C:\Users\<用户>\AppData\Local\Programs\GoGuo\bin\mihomo.exe
```

**配置部署模式**

首次启动 GoGuo 时，在向导页面选择：

- **部署模式**: `Windows + WSL`（Coordinated）
- **宿主平台**: Windows（程序自动检测）

数据目录结构：

```text
C:\Users\<用户>\AppData\Local\Programs\GoGuo\
  bin\
    mihomo.exe                 # 代理引擎
  data\
    baseline\                  # 网络状态快照
    config\
      settings.json            # 用户配置（含 deployment_mode: "coordinated"）
      site-definitions\        # 目标站点定义
    rules\
      current-rules.yaml       # mihomo 规则
    mihomo\
      config.yaml              # mihomo 运行配置
    audit\
      audit-*.jsonl            # 操作审计
```

## 4. 验证部署

### A4. 验证部署

**Windows 侧验证**

```powershell
# 启动 GoGuo
Start-Process "C:\Users\<用户>\AppData\Local\Programs\GoGuo\GoGuo.exe"

# 检查 mihomo 进程
Get-Process mihomo -ErrorAction SilentlyContinue

# 检查代理端口（默认 7890）
netstat -an | findstr "7890"
```

**WSL 侧验证**

```bash
# 在 WSL 中检查代理环境变量（应由 GoGuo 自动配置）
echo $http_proxy
echo $https_proxy

# 检查 Git 代理
git config --global --get http.proxy

# 测试目标站点可达性
curl -I --max-time 5 https://github.com
```

**协同一致性验证**

在 GoGuo UI 中：
1. 仪表盘页面应显示 "部署模式: Windows + WSL"
2. WSL 状态项应显示为绿色（已配置）
3. 修改 Windows 代理后，WSL 侧 `curl` 应同步生效

---

## 方向 B：WSL 单实例 → 管理 Win + WSL

在 WSL 上运行 GoGuo，通过 WindowsRemoteAdapter（`cmd.exe` / `powershell.exe` 桥接）管理 Windows 侧配置。

### B1. 前置条件

| 条件 | 检查命令 | 说明 |
|------|----------|------|
| WSL2 Ubuntu/Debian | `cat /etc/os-release` | 推荐 Ubuntu 22.04+ |
| Node.js 18+ | `node --version` | LTS 版本 |
| pnpm | `pnpm --version` | 包管理器 |
| Rust stable | `rustc --version` | Tauri 后端编译 |
| pkg-config + libwebkit2gtk | `pkg-config --modversion webkit2gtk-4.1` | Tauri Linux 后端依赖 |

**Windows 侧（被远程管理）**

| 条件 | 检查命令 | 说明 |
|------|----------|------|
| Windows 10/11 | — | 被远程管理侧无需安装 GoGuo |
| PowerShell 可从 WSL 调用 | `powershell.exe -Command "echo ok"` | 桥接操作依赖 |

**mihomo 二进制**

- 版本: mihomo v1.18.x（MetaKernel 分支）
- 平台: Linux amd64
- 下载: https://github.com/MetaCubeX/mihomo/releases

### B2. 构建步骤（WSL Bash）

```bash
# 进入项目目录
cd /home/slivelight/projects/goguo

# 安装前端依赖
pnpm install

# 构建 Linux 安装包（Tauri AppImage/Deb）
pnpm tauri build

# 输出位置
# src-tauri/target/release/bundle/deb/goguo_0.1.0_amd64.deb
# src-tauri/target/release/bundle/appimage/goguo_0.1.0_amd64.AppImage
```

### B3. 安装与配置

```bash
# 安装（Deb 方式）
sudo dpkg -i src-tauri/target/release/bundle/deb/goguo_0.1.0_amd64.deb

# 或直接运行 AppImage
chmod +x src-tauri/target/release/bundle/appimage/goguo_0.1.0_amd64.AppImage
./src-tauri/target/release/bundle/appimage/goguo_0.1.0_amd64.AppImage
```

**准备 mihomo 二进制**

```bash
# 创建 bin 目录
mkdir -p ~/.local/share/goguo/bin

# 下载 mihomo Linux 版本
# 从 https://github.com/MetaCubeX/mihomo/releases 下载
# mihomo-linux-amd64-v1.18.x.gz
gunzip mihomo-linux-amd64-v1.18.x.gz
chmod +x mihomo-linux-amd64
mv mihomo-linux-amd64 ~/.local/share/goguo/bin/mihomo
```

**配置部署模式**

首次启动 GoGuo 时，在向导页面选择：

- **部署模式**: `Windows + WSL`（Coordinated）
- **宿主平台**: WSL/Linux（程序自动检测）

数据目录结构：

```text
~/.local/share/goguo/
  bin/
    mihomo                          # 代理引擎（Linux 版）
  data/
    baseline/                       # 网络状态快照
    config/
      settings.json                 # 用户配置（含 deployment_mode: "coordinated"）
      site-definitions/             # 目标站点定义
    rules/
      current-rules.yaml            # mihomo 规则
    mihomo/
      config.yaml                   # mihomo 运行配置
    audit/
      audit-*.jsonl                 # 操作审计
```

### B4. 验证部署

**WSL 侧验证**

```bash
# 启动 GoGuo
goguo &

# 检查 mihomo 进程
ps aux | grep mihomo

# 检查代理端口（默认 7890）
ss -tlnp | grep 7890

# 测试代理可达性
curl -I --max-time 5 -x http://127.0.0.1:7890 https://github.com
```

**Windows 侧验证**

```powershell
# 检查 Windows 系统代理是否已被设置（由 WSL 侧 GoGuo 通过 powershell.exe 远程配置）
netsh winhttp show proxy

# 检查 Windows 代理环境变量
[System.Net.WebProxy]::GetDefaultProxy()

# 测试通过代理访问
curl -I --max-time 5 https://github.com
```

**协同一致性验证**

在 GoGuo UI 中：
1. 仪表盘页面应显示 "部署模式: Windows + WSL"
2. Windows 状态项应显示为绿色（已配置）
3. 修改 WSL 侧代理后，Windows 侧浏览器应同步生效

## 5. 常见问题

### Q1: WSL 侧写入 `/etc/resolv.conf` 失败

- 原因: 需要 root 权限
- 解决: GoGuo 会生成 sudo 命令，用户在 WSL 手动执行

### Q2: WSL 未运行时无法配置

- 原因: GoGuo 无法访问 WSL 进程
- 解决: 先启动 WSL（`wsl` 命令），再操作 GoGuo

### Q3: mihomo 进程无法启动

- 原因: 二进制路径错误或权限不足
- 解决: 确认 `bin\mihomo.exe` 存在且可执行

## 6. 回滚

```powershell
# 停止 GoGuo 服务（UI 中点击"停止服务"）

# 卸载程序
# 控制面板 → 程序 → 卸载 GoGuo

# 手动清理数据目录（可选）
Remove-Item -Recurse "C:\Users\<用户>\AppData\Local\Programs\GoGuo"
```

## 7. 附录

### 7.1 数据目录权限

- Windows 侧: 用户级权限（`%LOCALAPPDATA%`）
- WSL 侧: `/etc/resolv.conf`、`/etc/environment` 写入需 sudo
- WSL→Windows 桥接: 注册表、hosts 文件操作通过 `powershell.exe` 执行，继承 Windows 用户权限

### 7.2 WSL2 网络模式

GoGuo 自动检测 WSL2 网络模式，并按部署方向适配：

| 网络模式 | 方向 A（Win 单实例） | 方向 B（WSL 单实例） |
|----------|---------------------|---------------------|
| NAT（默认） | 代理地址设为 WSL 网关 IP；WSL 代理环境变量显式配置 | mihomo 绑定 `0.0.0.0`；Windows 代理指向 WSL IP |
| Mirrored（镜像） | 共享 localhost；先验证可达性 | 共享 localhost；先验证可达性 |

检查当前模式：

```powershell
# Windows 中检查 .wslconfig
cat $env:USERPROFILE\.wslconfig
# networkingMode=mirrored → 镜像模式
```

### 7.3 两种方向的适用场景

| 场景 | 推荐方向 | 原因 |
|------|----------|------|
| 开发者主力在 WSL | 方向 B（WSL 单实例） | 开发工具链在 WSL，减少切换 |
| 办公用户主力在 Windows | 方向 A（Win 单实例） | 系统代理原生集成更好 |
| 需要系统级代理图标/通知 | 方向 A（Win 单实例） | Windows 系统托盘集成 |
| 需要轻量后台服务 | 方向 B（WSL 单实例） | Linux 进程管理更轻量 |