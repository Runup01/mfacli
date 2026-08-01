# mfacli 🐾

<p align="center">
  <a href="https://github.com/Runup01/mfacli/actions/workflows/ci.yml"><img src="https://github.com/Runup01/mfacli/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="https://github.com/Runup01/mfacli/releases"><img src="https://img.shields.io/github/v/release/Runup01/mfacli?style=flat-square" alt="Release"/></a>
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square" alt="Rust"/>
  <img src="https://img.shields.io/badge/Platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=flat-square" alt="Platforms"/>
  <img src="https://img.shields.io/badge/License-MIT-yellow?style=flat-square" alt="MIT"/>
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> ·
  <a href="#安装">安装</a> ·
  <a href="#使用">使用</a> ·
  <a href="#环境变量">环境变量</a> ·
  <a href="#系统依赖">依赖</a> ·
  <a href="#安全设计">安全</a> ·
  <a href="#常见问题">FAQ</a>
</p>

---

**mfacli** 是一款面向开发者的**本地优先 MFA / OTP 管理工具**。它以单个静态二进制运行，无需联网、无需云端账号，即可在终端中安全地生成、管理和复制两步验证码（TOTP / HOTP / Steam Guard）。

除了可被脚本与 CI 直接调用的**命令行界面**，它还提供一个**交互式 TUI**：实时代码刷新、终端二维码同步、鼠标双击一键复制，以及像素宠物、天气、中国黄历等可开关的趣味装饰。所有密钥默认以明文（权限 `600`）或 **AES-256-GCM + Argon2id** 加密形式存储在你自己的机器上，永不外传。

```
  今日时间:  2026-08-02 周日 10:15:33  |  Shanghai ☀️ +34°C
  今日八字:  丙午年丙申月戊戌日 平日    |  宜:祭祀 出行  忌:动土 开仓

  NAME          ISSUER        CODE       ⏱
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  github        GitHub        482916   ██████░░░░ 18s
  aws           AWS           917304   ███░░░░░░░ 9s
```

---

## 功能特性

**🔐 安全且本地优先**
- 密钥存储于本地，**零云端同步、零遥测、零网络依赖**（核心 OTP 为纯本地计算）
- 可选**应用锁**：AES-256-GCM 认证加密 + Argon2id 内存硬密钥派生，启用前强制明文备份
- Vault 文件权限 `600`；所有密码 / 密钥经隐藏输入（`rpassword`），不进入 shell 历史

**⌨️ 命令行 + 交互式 TUI 双形态**
- CLI 输出纯 stdout，可直接管道、嵌入脚本与 CI（`mfa code github` → `482916`）
- TUI 实时代码刷新 + 倒计时进度条，支持**鼠标双击复制**、键盘全功能管理
- 复制成功底栏即时提示 `✓ 已复制 … 可粘贴`，宠物同步切换开心表情

**🎯 协议全覆盖**
- **TOTP**（RFC 6238）、**HOTP**（RFC 4226）、**Steam Guard**
- 支持 SHA1 / SHA256 / SHA512，6 / 8 位验证码，自定义时间周期

**📱 二维码双向**
- `mfa show` 在终端渲染二维码，供手机 Authenticator 扫描同步
- `mfa scan` 解码 QR 图片直接导入条目

**🔄 导入 / 导出全格式**
- 导入自动检测：`otpauth://`、JSON、CSV、Google Authenticator 迁移；加密备份需显式指定
- 导出三档：`otpauth`（通用迁移）、`json`（完整保真）、`encrypted`（密码保护备份）
- 名称冲突自动重命名，永不覆盖已有数据；完整支持中文 / 特殊符号

**🌏 跨平台 + 原生包**
- 单文件静态二进制，**无运行时依赖**
- 提供 **DEB / RPM / tarball / zip**，覆盖 macOS、Windows、Debian/Ubuntu、CentOS/Rocky/AlmaLinux（x86_64 + ARM64）

**🀄 细节友好**
- CJK 名称按显示宽度对齐，截断不破坏字符边界
- 条目按发行方 / 名称排序，列宽自适应

**🐾 可选趣味装饰**（均可关闭，绝不阻塞核心逻辑）
- 5 种像素宠物（robot / dino / cat / ghost / dragon）带情绪动画
- 天气（自动 IP 定位 + 30 分钟缓存 + 离线降级）
- 中国黄历：天干地支 + 建除十二神 + 宜忌（纯离线计算）

---

## 为什么选择 mfacli

| 能力 | mfacli | Authy | Google Auth | 1Password |
|---|:---:|:---:|:---:|:---:|
| CLI 可脚本化 / 可嵌入 CI | ✅ | ❌ | ❌ | ❌ |
| 交互式 TUI + 鼠标双击复制 | ✅ | ❌ | ❌ | ❌ |
| 完全离线、零云依赖 | ✅ | ❌ | ✅ | ❌ |
| 开源 + 加密存储 | ✅ | 部分 | ❌ | ✅ |
| 单文件、零运行时依赖 | ✅ | ❌ | ❌ | ❌ |
| DEB / RPM 原生包 | ✅ | ❌ | ❌ | ❌ |
| 终端二维码显示 / 扫描 | ✅ | ✅ | ✅ | ✅ |

---

## 安装

mfacli 是**静态编译的单文件**，安装即把 `mfa` 二进制放进 `PATH`，无需运行时、无需依赖库。当前版本 **0.1.3**。

### 一行安装（按平台）

```bash
# macOS (Apple Silicon) — tarball 文件名不含版本号，latest 链接稳定
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-aarch64-apple-darwin.tar.gz
tar xzf mfacli-aarch64-apple-darwin.tar.gz && chmod +x mfa && sudo mv mfa /usr/local/bin/

# macOS (Intel)：把上面 aarch64 换成 x86_64 即可

# Debian / Ubuntu (amd64) — 文件名含版本号，升级时改 0.1.3
curl -LO https://github.com/Runup01/mfacli/releases/download/v0.1.3/mfacli_0.1.3_amd64.deb
sudo dpkg -i mfacli_0.1.3_amd64.deb

# CentOS / Rocky / AlmaLinux (x86_64)
curl -LO https://github.com/Runup01/mfacli/releases/download/v0.1.3/mfacli-0.1.3-1.x86_64.rpm
sudo rpm -i mfacli-0.1.3-1.x86_64.rpm

# Windows：下载 mfacli-x86_64-pc-windows-msvc.zip，解压得 mfa.exe，放入 PATH 目录
```

安装后验证：

```bash
mfa --version      # 应输出 mfa 0.1.3
mfa --help         # 查看全部子命令
```

<details>
<summary><b>📦 全平台下载矩阵</b></summary>

| 平台 | 架构 | 文件 | 安装方式 |
|------|------|------|---------|
| macOS | Apple Silicon (M1–M4) | `mfacli-aarch64-apple-darwin.tar.gz` | 解压 → `chmod +x` → 移入 PATH |
| macOS | Intel | `mfacli-x86_64-apple-darwin.tar.gz` | 同上 |
| Linux | x86_64 | `mfacli-x86_64-unknown-linux-gnu.tar.gz` | 解压 → 移入 PATH，或用下方 deb/rpm |
| Linux | ARM64 | `mfacli-aarch64-unknown-linux-gnu.tar.gz` | 同上 |
| Windows | x86_64 | `mfacli-x86_64-pc-windows-msvc.zip` | 解压 → `mfa.exe` 放入 PATH |
| Debian/Ubuntu | amd64 | `mfacli_<ver>_amd64.deb` | `sudo dpkg -i` |
| Debian/Ubuntu | arm64 | `mfacli_<ver>_arm64.deb` | `sudo dpkg -i` |
| CentOS/Rocky/Alma | x86_64 | `mfacli-<ver>-1.x86_64.rpm` | `sudo rpm -i` |
| CentOS/Rocky/Alma | aarch64 | `mfacli-<ver>-1.aarch64.rpm` | `sudo rpm -i` |

> tarball 内除 `mfa` 外还附带 `README.md` 与 `LICENSE`。deb/rpm 会把二进制装到 `/usr/local/bin/mfa`，文档装到 `/usr/share/doc/mfacli/`。

</details>

<details>
<summary><b>🍎 macOS 细节：PATH / 执行权限 / Gatekeeper</b></summary>

- **执行权限**：tarball 解压后若提示 `Permission denied`，执行 `chmod +x mfa`。
- **Gatekeeper 拦截**（“无法打开，因为无法验证开发者”）：下载文件带隔离属性，移除即可：
  ```bash
  xattr -d com.apple.quarantine /usr/local/bin/mfa
  # 或首次右键 → 打开 → 在弹窗中确认
  ```
- **Apple Silicon 误装 Intel 包**：会报 `exec format error`，请用 `uname -m` 确认（`arm64` 选 aarch64 包，`x86_64` 选 x86_64 包）。
- **Homebrew 用户**：暂无官方 tap，可直接把二进制软链到 brew 的 bin：`ln -s /usr/local/bin/mfa /opt/homebrew/bin/mfa`（Apple Silicon 的 brew 在 `/opt/homebrew`）。

</details>

<details>
<summary><b>🐧 Linux 细节：包管理 / SELinux / 架构</b></summary>

各发行版安装剪贴板依赖（详见[系统依赖](#系统依赖)）后，包本身**无其他运行时依赖**。

```bash
# Debian / Ubuntu 全家桶
sudo apt update && sudo apt install -y ./mfacli_0.1.3_amd64.deb

# RHEL / CentOS 7（yum）
sudo yum install -y ./mfacli-0.1.3-1.x86_64.rpm

# CentOS 8+ / Rocky / Alma / Fedora（dnf）
sudo dnf install -y ./mfacli-0.1.3-1.x86_64.rpm

# 已装后升级
sudo dpkg -i ./mfacli_0.1.3_amd64.deb     # deb 直接覆盖
sudo rpm -U ./mfacli-0.1.3-1.x86_64.rpm    # rpm 用 -U 升级
```

- **架构核对**：`uname -m` → `x86_64` 选 x86_64 包，`aarch64` 选 aarch64 包。
- **SELinux / AppArmor**：mfacli 仅读写用户配置目录与调用本地剪贴板程序，默认策略下无需额外放行。
- **无 root 安装**：tarball 方式把 `mfa` 放到 `~/.local/bin`（并确保其在 `PATH`）即可，无需 sudo。

</details>

<details>
<summary><b>🪟 Windows 细节：PATH / Defender / 终端</b></summary>

1. 解压 `mfacli-x86_64-pc-windows-msvc.zip` 得到 `mfa.exe`。
2. 放入固定目录，如 `C:\tools\mfacli`。
3. 加入 PATH（PowerShell，无需管理员）：
   ```powershell
   [Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\tools\mfacli", "User")
   ```
   重开终端后 `mfa --version` 生效。或图形方式：设置 → 系统 → 关于 → 高级系统设置 → 环境变量 → 编辑用户 `Path`。
4. **Windows Defender 误报**：小众 Rust 二进制偶发 SmartScreen/Defender 拦截，可对该文件“允许”或加白名单 `C:\tools\mfacli`。
5. **终端**：推荐 Windows Terminal（自带 UTF-8 与 Unicode，TUI/中文/二维码显示最佳）；老 `cmd` 中文可能乱码，见[常见问题](#常见问题)。

</details>

<details>
<summary><b>🛠 从源码编译</b></summary>

**前置**：[Rust ≥ 1.75](https://rustup.rs/)（`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）。

```bash
git clone https://github.com/Runup01/mfacli.git && cd mfacli

# 方式 A：cargo install（装到 ~/.cargo/bin，确保该目录在 PATH）
cargo install --path .

# 方式 B：手动构建 release 并放置
cargo build --release
sudo cp target/release/mfa /usr/local/bin/     # Linux/macOS
# Windows：copy target\release\mfa.exe C:\tools\mfacli\
```

**国内加速**（crates.io 慢时配置清华镜像）：

```bash
mkdir -p ~/.cargo && cat >> ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = 'tuna'

[source.tuna]
registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
EOF
```

**校验完整性**：每个 Release 资源在 GitHub 页面附带 `sha256`，下载后可核对：

```bash
shasum -a 256 mfacli-aarch64-apple-darwin.tar.gz   # 与页面 sha256 比对
# Linux: sha256sum <file>     Windows(certutil): certutil -hashfile <file> SHA256
```

</details>

---

## 使用

### 快速上手

```bash
mfa add github --issuer GitHub     # 添加条目（密钥隐藏输入，不进历史）
mfa code github                    # 获取验证码 → 482916（纯 stdout）
mfa code github -c                 # 获取并复制到剪贴板
mfa copy github                    # 仅复制到剪贴板
mfa list                           # 列出全部（含时间 / 天气 / 黄历）
mfa show github                    # 详情 + 终端二维码（手机扫描同步）
mfa scan qr.png                    # 扫描 QR 图片导入
mfa tui                            # 启动交互式 TUI
```

### 命令一览

| 命令 | 说明 |
|------|------|
| `mfa` | 无参数等同 `mfa list` |
| `mfa init [--encrypt]` | 初始化 vault（通常无需手动；`--encrypt` 直接加密模式） |
| `mfa add <name> [-s SECRET] [-i ISSUER] [-a ALGO] [-d DIGITS] [-p PERIOD]` | 添加条目；省略 `-s` 则隐藏交互输入 |
| `mfa code <name> [-c]` | 输出当前验证码；`-c` 同时复制 |
| `mfa copy <name>` | 复制当前验证码到剪贴板 |
| `mfa list [-l N] [--all]` | 列出条目（默认按终端高度分页） |
| `mfa show <name>` | 显示密钥 / 算法 / URI + 终端二维码 |
| `mfa scan <img> [-n NAME]` | 解码 QR 图片（PNG/JPG）并导入 |
| `mfa edit <name> [-r NEW] [-s] [-i ISS]` | 改名称 / 密钥 / 发行方；`-s` 不带值则隐藏输入新密钥 |
| `mfa rename <old> <new>` | 重命名（`edit --rename` 的快捷方式） |
| `mfa remove <name>` | 删除（有确认） |
| `mfa export [-o FILE] [-f FMT]` | 导出；`FMT` = `otpauth`(默认) / `json` / `encrypted` |
| `mfa import <file> [-s FMT]` | 导入；省略 `-s` 自动检测（`encrypted` 除外，须显式指定） |
| `mfa lock [--backup PATH]` | 启用应用锁（强制先备份 → 设密码两次确认） |
| `mfa unlock` | 禁用应用锁（验证密码后解密回明文） |
| `mfa config [OPTIONS]` | 查看 / 修改 TUI 配置 |
| `mfa tui` | 启动交互式 TUI |

**`add` / `edit` 可选参数默认值**：`-a` 算法 `SHA1`；`-d` 位数 `6`；`-p` 周期 `30` 秒。算法可选 `SHA1` / `SHA256` / `SHA512`。

**`config` 参数**（开关类需带显式值 `true`/`false`）：

| 参数 | 取值 | 说明 |
|------|------|------|
| `--pet` | `robot` / `dino` / `cat` / `ghost` / `dragon` | 像素宠物风格 |
| `--city` | 城市名（如 `Beijing`） | 天气城市；不传则维持现状，自动定位由“不设 city”实现 |
| `--show-weather` | `true` / `false` | 天气开关 |
| `--show-bazi` | `true` / `false` | 黄历开关 |
| `--show-pet` | `true` / `false` | 宠物开关 |

```bash
mfa config                       # 仅查看当前配置（含是否已加密）
mfa config --pet dino --show-weather false
```

<details>
<summary><b>🧩 典型场景</b></summary>

**① 脚本 / CI 取码（无交互）**

```bash
# 明文 vault：直接取码
CODE=$(mfa code github)
curl -H "X-OTP: $CODE" https://api.example.com/deploy

# 加密 vault：用环境变量喂密码（不进 history）
CODE=$(MFA_PASSWORD="$VAULT_PW" mfa code github)
```

**② 配合 ssh / 自动登录填码**

```bash
# 例：把当前码写入剪贴板后手动粘贴，或拼进 expect/自动化脚本
mfa copy jumpserver && echo "code copied, paste when prompted"
```

**③ 从其他工具迁移**

```bash
# Google Authenticator：导出二维码 → 截图/保存为 png → 逐个 scan
mfa scan google-export-1.png

# 或导出为 otpauth 文本（每行一个 URI）后批量导入
mfa import tokens.txt            # 自动识别 otpauth 格式

# Authy / 其他：导出 otpauth:// 列表或 JSON 后
mfa import backup.json           # 自动识别 json
```

**④ 备份与恢复**

```bash
mfa export -f json -o /tmp/mfa-backup.json        # 完整保真备份（明文 JSON，请妥善保管）
mfa export -f encrypted -o /tmp/mfa-backup.enc    # 加密备份（可放云端，需密码恢复）
# 恢复
mfa import /tmp/mfa-backup.json
mfa import -s encrypted /tmp/mfa-backup.enc       # 加密备份恢复须显式 -s
```

**⑤ 应用锁完整流程**

```bash
mfa lock --backup /secure/usb/mfa-plain.json   # 1) 强制写出明文逃生备份
# 2) 设密码（输入两次；<8 位会额外警告）
# 之后每次 mfa 访问都需密码（交互输入 或 MFA_PASSWORD）
mfa unlock                                      # 验证密码后解密回明文 vault.json
```

> ⚠️ 忘记锁密码 = 加密 vault 不可恢复（AES-GCM 无后门）。务必保管 `--backup` 那份明文文件。

</details>

### TUI 快捷键

```bash
mfa tui
```

| 键 | 功能 | 键 | 功能 |
|---|---|---|---|
| `↑↓` / `jk` | 导航 | `a` | 添加 |
| `c` / `Enter` | 复制验证码 | `e` | 编辑（名称/发行方/密钥子菜单） |
| 🖱 双击 | 复制验证码 | `r` | 重命名 |
| `v` | 查看二维码 | `d` | 删除 |
| `s` / `Tab` | 设置弹窗 | `q` / `Esc` | 退出 / 取消 |

<details>
<summary><b>🎮 TUI 进阶</b></summary>

- **鼠标双击复制**：在列表行上 400ms 内连点两次左键，自动选中该行并复制验证码，底栏提示 `✓ 已复制 … 可粘贴`。
- **设置弹窗**（`s`）：用 `↑↓` 选项目、`Enter` 切换/进入，可改宠物、开关天气/黄历/宠物、设城市、触发导入/导出、查看加密状态。`Esc` 关闭。
- **编辑子菜单**（`e`）：`n` 改名称、`i` 改发行方、`s` 改密钥（隐藏输入）。
- **二维码覆盖层**（`v`）：居中显示当前条目 QR，供手机扫描；`Esc`/`v`/`Enter` 关闭。
- **实时刷新**：代码与倒计时每 0.5s 刷新；剩余 ≤5s 变红提醒。
- **装饰不阻塞**：宠物/天气/黄历均为后台或纯本地计算，绝不拖慢取码。

</details>

---

## 环境变量

mfacli 只识别**一个**环境变量：

| 变量 | 作用 | 何时读取 | 何时**不**读取 |
|------|------|---------|--------------|
| `MFA_PASSWORD` | 加密 vault 的解锁密码 | 加载加密 vault（`vault.enc` 存在时）、`mfa import -s encrypted` | 明文 vault 场景；`mfa lock` 设密码时（必须交互二次确认，防误设） |

设置后跳过交互式密码提示，适用于脚本与 CI：

```bash
# 一次性（仅当前命令，最安全，不进 history）
MFA_PASSWORD="xxx" mfa code github

# 当前 shell 会话
export MFA_PASSWORD="xxx"
mfa list

# CI / CD（GitHub Actions 示例）
env:
  MFA_PASSWORD: ${{ secrets.MFA_VAULT_PASSWORD }}
```

<details>
<summary><b>🔒 安全使用 MFA_PASSWORD 的细节</b></summary>

- **进程可见性**：`export` 形式下，密码会出现在进程环境（Linux 的 `/proc/<pid>/environ`、macOS 的 `ps eww`）。**多用户共享主机**上，他人可能读到——此场景请改用交互式输入，或收紧 `/proc` 可见性（`hidepid=2`）。
- **避免进 history**：不要 `export MFA_PASSWORD=明文` 后回车（会被记入 `~/.bash_history`/`~/.zsh_history`）。推荐：
  ```bash
  read -s -p "vault password: " MFA_PASSWORD && export MFA_PASSWORD   # 隐藏输入，不写 history
  # 或用权限 600 的 env 文件：set -a; . ~/.config/mfa-cli/.env; set -a   （该文件 chmod 600）
  ```
- **命令内联最干净**：`MFA_PASSWORD="x" mfa ...` 这种写法密码只存在于该子进程，命令结束后即消失，且通常不被 history 记录明文（取决于 shell 配置）。
- **与 lock 的关系**：`mfa lock` 设密码**不读** `MFA_PASSWORD`（强制交互确认）；设好之后，日常访问才用 `MFA_PASSWORD` 解锁。

</details>

---

## 系统依赖

**核心功能（OTP 生成 / 存储 / TUI / 二维码 / 黄历）零外部依赖。** 仅以下**可选**能力需要系统组件，缺失时 mfacli 会优雅降级而非崩溃：

| 能力 | 是否必需 | 缺失时行为 |
|------|:-------:|-----------|
| 剪贴板复制 | 否 | `mfa code` 仍输出到 stdout；`copy`/TUI 复制报错提示 |
| 天气显示 | 否 | 不显示天气行；OTP 等主功能不受影响 |
| UTF-8 locale | 建议 | 中文名称/黄历可能乱码（见下） |
| 等宽 + Unicode 字体 | 建议 | TUI 边框/进度条/二维码观感变差 |

### 剪贴板工具（按平台）

| 系统 | 工具 | 说明 |
|------|------|------|
| macOS | `pbcopy` | 系统内置，无需安装 |
| Windows | `clip` | 系统内置，无需安装 |
| Linux X11 | `xclip` 或 `xsel` | mfacli 按 `xclip → xsel → wl-copy` 顺序自动尝试 |
| Linux Wayland | `wl-clipboard` | 提供 `wl-copy` |

各发行版安装命令：

```bash
# Debian / Ubuntu
sudo apt install -y xclip            # X11
sudo apt install -y wl-clipboard     # Wayland

# RHEL / CentOS 7
sudo yum install -y xclip

# CentOS 8+ / Rocky / Alma / Fedora
sudo dnf install -y xclip            # 或 wl-clipboard

# Arch
sudo pacman -S xclip wl-clipboard

# openSUSE
sudo zypper install -y xclip

# macOS（一般无需；若用 Homebrew 环境也想统一）
brew install --cask ...   # 不需要，pbcopy 已内置
```

> **纯 SSH 无图形会话**（无 X11 转发 / 无 Wayland）时，剪贴板工具无法工作——这是预期行为；此时用 `mfa code <name>` 取码再手动输入即可。

### 天气的网络要求

- 天气取自 **wttr.in**（HTTP/80），后台线程拉取，**30 分钟缓存**，超时/失败**静默不显示**，绝不阻塞取码。
- 连通性探测**国内 DNS 优先**（`223.5.5.5` / `114.114.114.114` 先于 `8.8.8.8` / `1.1.1.1`），但实际拉取仍连 `wttr.in`。
- 因此**国内若无法访问 wttr.in，天气行将不显示**，属正常降级；用 `mfa config --show-weather false` 可彻底关闭以避免每次启动的探测等待。

### 中文 / UTF-8 locale（Linux 重点）

TUI 与列表的中文名称、黄历依赖 UTF-8。若 Linux 下中文乱码：

```bash
locale -a | grep -i utf            # 查看可用 UTF-8 locale
export LANG=zh_CN.UTF-8            # 或 en_US.UTF-8
export LC_ALL=zh_CN.UTF-8
# 持久化：写入 ~/.bashrc / ~/.zshrc，或用 sudo locale-gen zh_CN.UTF-8（Debian 系）
```

Windows 建议用 **Windows Terminal**；老 `cmd` 可执行 `chcp 65001` 切到 UTF-8。

---

## 文件位置

数据全部存于用户配置目录（不污染仓库、不联网上传）：

| 系统 | 目录 |
|------|------|
| macOS | `~/Library/Application Support/mfa-cli/` |
| Linux | `${XDG_CONFIG_HOME:-~/.config}/mfa-cli/` |
| Windows | `%APPDATA%\mfa-cli\` |

| 文件 | 说明 | 权限 |
|------|------|------|
| `vault.json` | 明文 vault | 600 |
| `vault.enc` | 加密 vault（启用应用锁后） | 600 |
| `config.json` | 用户偏好（宠物 / 天气 / 黄历 / 城市） | 644 |
| `weather_cache.txt` | 天气缓存（首行时间戳，30 分钟有效） | 644 |

> 迁移机器：复制整个 `mfa-cli/` 目录即可；加密 vault 需连同密码一起迁移。

---

## 安全设计

- **加密方案**：AES-256-GCM（认证加密）+ Argon2id（内存硬密钥派生），每次加密独立随机 salt / nonce
- **无后门**：忘记密码即不可恢复——这是设计，故 `mfa lock` 强制先备份
- **零网络**：OTP 生成为纯本地数学运算；天气为唯一联网项，可关闭且离线自动降级
- **零遥测**：不收集任何数据，不云同步
- **隐藏输入**：密码 / 密钥经 `rpassword`，无回显、无历史

详见 [SECURITY.md](SECURITY.md)。发现漏洞请邮件 **bdstravel@126.com**，勿公开提交 Issue。

---

## 常见问题

<details>
<summary><b>❓ 点击展开 FAQ</b></summary>

- **`mfa: command not found`**：二进制不在 `PATH`。确认 `which mfa` / `where mfa`，或把所在目录加入 `PATH` 后重开终端。
- **macOS “无法打开 / 无法验证开发者”**：`xattr -d com.apple.quarantine /usr/local/bin/mfa`，或右键→打开→确认。
- **`exec format error` / 架构不对**：用 `uname -m` 核对，ARM Mac 装 aarch64 包、Intel 装 x86_64 包。
- **中文乱码**：Linux 设 `LANG`/`LC_ALL` 为 `*.UTF-8`；Windows 用 Windows Terminal 或 `chcp 65001`。
- **复制没反应**：Linux 缺剪贴板工具（装 `xclip`/`wl-clipboard`），或处于无图形的纯 SSH 会话（此时改用 `mfa code`）。
- **天气不显示**：无法访问 wttr.in（国内常见），属正常降级；可 `--show-weather false` 关闭。
- **TUI 二维码花屏**：终端字体不支持半块字符；改用 `mfa show <name>` 或换等宽 Unicode 字体。
- **应用锁忘密码**：加密 vault 不可解密；用 `mfa lock` 时生成的明文备份 `mfa import <备份.json>` 恢复数据。
- **导入提示格式不对**：`encrypted` 不会被自动检测，须 `mfa import -s encrypted <file>`；其余格式自动识别。

</details>

---

## 开发

```bash
cargo build              # 开发构建
cargo build --release    # 发布构建
cargo test               # 单元测试
cargo clippy             # 静态检查（CI 要求零警告）
```

贡献指南见 [CONTRIBUTING.md](CONTRIBUTING.md)，版本记录见 [CHANGELOG.md](CHANGELOG.md)，导入模板见 [`examples/`](examples/)。

---

## 许可证

本项目基于 [MIT License](LICENSE) 开源，© 2026 Runup01。

## 作者

**Runup01** — [GitHub](https://github.com/Runup01)

如果 mfacli 对你有帮助，欢迎给仓库点个 ⭐，也欢迎提交 Issue 与 Pull Request。
