<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo-dark.svg">
    <img alt="mfacli" src="assets/logo-light.svg" width="520"/>
  </picture>
</p>

<p align="center">
  <a href="https://github.com/Runup01/mfacli/actions/workflows/ci.yml"><img src="https://github.com/Runup01/mfacli/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="https://github.com/Runup01/mfacli/releases"><img src="https://img.shields.io/github/v/release/Runup01/mfacli?style=flat-square" alt="Release"/></a>
  <a href="https://github.com/Runup01/mfacli/releases"><img src="https://img.shields.io/github/downloads/Runup01/mfacli/total?style=flat-square&label=downloads" alt="Downloads"/></a>
  <img src="https://img.shields.io/badge/Platforms-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=flat-square" alt="Platforms"/>
  <img src="https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square" alt="Rust"/>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow?style=flat-square" alt="MIT"/></a>
</p>

<h1 align="center">mfacli 🐾</h1>

<p align="center">
  <b>本地优先的开发者 MFA / OTP 管理工具</b> — 终端里的第二步验证<br/>
  <sub>A local-first, developer-friendly MFA/OTP manager — CLI + TUI, encrypted storage, zero cloud.</sub>
</p>

<p align="center">
  <a href="#快速开始">快速开始</a> ·
  <a href="docs/install.md">安装</a> ·
  <a href="docs/usage.md">使用指南</a> ·
  <a href="docs/faq.md">FAQ</a> ·
  <a href="#安全设计">安全</a> ·
  <a href="CHANGELOG.md">更新日志</a>
</p>

---

**mfacli** 把两步验证码放进你的终端：单个静态二进制，**零云端、零遥测、零运行时依赖**。命令行可管道、可嵌入脚本与 CI；交互式 TUI 支持实时代码、鼠标双击复制、终端二维码同步，还有一只可选的像素宠物 🐾。密钥只存在你自己的机器上，可选 **AES-256-GCM + Argon2id** 应用锁。

> **双形态**：直接运行 `mfa` 是 **命令行**（默认，纯 stdout，可管道、可入 CI）；运行 `mfa tui` 进入 **交互式 TUI**（实时代码、双击复制、二维码、宠物）。

**⌨️ 命令行（默认）**

```
  ┌──────────────────────────────────────────────────────────────┐
  │ 2026-08-03 周一 10:30:00 │ Shanghai ☀️ +34°C                  │
  │ 丙午年丙酉月己亥日 满日  │ 宜:祭祀 出行  忌:动土 开仓         │
  └──────────────────────────────────────────────────────────────┘

  ─────────────────────────────────────────────────────────────────────────────
  INDEX  NAME              ISSUER        CODE          ⏱     ADDED
  ─────────────────────────────────────────────────────────────────────────────
     01  github            GitHub        482916        18s   2026-06-01
     02  aws            ✦  AWS           917304        9s    2026-08-03
     03  jumpserver        JumpServer    204871        25s   2026-06-12
  ─────────────────────────────────────────────────────────────────────────────
  tip  mfa copy <name|index>  mfa show <name|index>  mfa tui
```

**🖥 交互式 TUI** — 运行 `mfa tui`

```
  ┌───────────────────────────────────────────────────────────────────────────┐
  │ 2026-08-03 周一 10:30:00  ┃  丙午年丙申月己亥日 满日  ┃  Shanghai ☀️ +34°C│
  ├───────────────────────────────────────────────────────────────────────────┤
  │  ┌───                                                                     │
  │  │◉ ◉│  [robot]  宜:祭祀 出行  忌:动土 开仓                               │
  │  │ ▽ │                                                                    │
  │ /└─┬─┘\                                                                   │
  │   │ │                                                                     │
  │   ┘ └                                                                     │
  ├───────────────────────────────────────────────────────────────────────────┤
  │ Tokens                                                                    │
  │ ▸ 01  github            GitHub     482916   ●●●●●●○○○○  18s               │
  │   02  aws           ✦ AWS        917304   ●●●○○○○○○○   9s                 │
  │   03  jumpserver        JumpServer 204871   ●●●●●●●●○○  25s               │
  ├───────────────────────────────────────────────────────────────────────────┤
  │ c 复制  a 添加  e 编辑  r 重命名  v 二维码  d 删除  s 设置  q 退出        │
  └───────────────────────────────────────────────────────────────────────────┘
```

---

## 🧩 功能介绍

| 功能 | 说明 |
|------|------|
| **命令行（默认）** | 直接运行 `mfa` 即 list；纯 stdout，可管道、嵌入脚本与 CI |
| **交互式 TUI** | `mfa tui` 进入：实时倒计时、双击复制、终端二维码、像素宠物 |
| **TOTP / HOTP / Steam Guard** | RFC 6238 / 4226，SHA1/256/512，6/8 位，自定义周期 |
| **二维码双向** | `show` 渲染 QR 供手机扫描；`scan` 扫 QR 图片导入 |
| **导入 / 导出** | otpauth / JSON / CSV / Google 迁移；加密备份可上云 |
| **批量扫码** | `scan` 支持目录递归批量导入 + 正则过滤 |
| **一键运维** | `backup` 时间戳备份 / `clear` 清空（自动备份+确认）/ `config --reset` |
| **加入时间** | 记录带 ADDED 日期列，7 天内新记录挂 ✦ 标签 |
| **应用锁** | AES-256-GCM + Argon2id，启用前强制备份 |
| **本机免密（opt-in）** | 密码托管 macOS Keychain / Windows DPAPI / Linux Secret Service，首次解锁后零输入 |
| **本地优先** | 零云端、零遥测，核心全离线；vault 权限 600 |
| **跨平台包** | 单静态二进制 + DEB / RPM / tarball / zip |
| **CJK 友好** | 中文名称按显示宽度对齐，截断不破坏字符 |
| **趣味装饰** | 5 种像素宠物、天气、中国黄历，均可关闭 |

## 🆚 对比

| 能力 | mfacli | Authy | Google Auth | 1Password |
|---|:---:|:---:|:---:|:---:|
| CLI 可脚本化 / 可嵌入 CI | ✅ | ❌ | ❌ | ❌ |
| 交互式 TUI + 鼠标双击复制 | ✅ | ❌ | ❌ | ❌ |
| 完全离线、零云依赖 | ✅ | ❌ | ✅ | ❌ |
| 开源 + 加密存储 | ✅ | 部分 | ❌ | ✅ |
| 系统钥匙串本机免密 | ✅（opt-in） | ❌ | ❌ | ✅ |
| 单文件、零运行时依赖 | ✅ | ❌ | ❌ | ❌ |
| DEB / RPM 原生包 | ✅ | ❌ | ❌ | ❌ |

## 🚀 快速开始

### 1) 安装

**🍎 macOS（Apple Silicon）**

```bash
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-aarch64-apple-darwin.tar.gz
tar xzf mfacli-aarch64-apple-darwin.tar.gz && chmod +x mfa && sudo mv mfa /usr/local/bin/
```

**🐧 Linux 通用（推荐：musl 静态包，全发行版通吃）**

```bash
# x86_64
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-x86_64-unknown-linux-musl.tar.gz
tar xzf mfacli-x86_64-unknown-linux-musl.tar.gz && chmod +x mfa && sudo mv mfa /usr/local/bin/

# ARM64
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-aarch64-unknown-linux-musl.tar.gz
tar xzf mfacli-aarch64-unknown-linux-musl.tar.gz && chmod +x mfa && sudo mv mfa /usr/local/bin/
```

> musl 版静态链接、零依赖，CentOS 7 / Rocky 8 / 新 glibc 发行版都能直接跑（deb/rpm 的 gnu 版在老系统可能缺 GLIBC）。

**🐧 Ubuntu / Debian（amd64，偏好包管理）**

```bash
curl -LO https://github.com/Runup01/mfacli/releases/download/v0.1.8/mfacli_0.1.8_amd64.deb
sudo dpkg -i mfacli_0.1.8_amd64.deb
```

**🎩 CentOS / Rocky / AlmaLinux（x86_64，偏好包管理）**

```bash
curl -LO https://github.com/Runup01/mfacli/releases/download/v0.1.8/mfacli-0.1.8-1.x86_64.rpm
sudo rpm -i mfacli-0.1.8-1.x86_64.rpm
```

**🪟 Windows（PowerShell）**

```powershell
Invoke-WebRequest https://github.com/Runup01/mfacli/releases/latest/download/mfacli-x86_64-pc-windows-msvc.zip -OutFile mfacli.zip
Expand-Archive mfacli.zip C:\tools\mfacli
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\tools\mfacli", "User")  # 重开终端生效
```

> Intel Mac / ARM64 Linux / 校验和 / 源码编译 → [docs/install.md](docs/install.md)

### 2) 使用

```bash
mfa add github --issuer GitHub   # 添加条目（密钥隐藏输入，不进历史）
mfa code github                  # → 482916（纯 stdout，可管道）
mfa copy github                  # 复制到剪贴板
mfa remove 02                    # 名称或列表序号 (01/02…) 都能定位条目
mfa scan ./qr-dir/ -f 'aliyun'   # 批量扫目录二维码 + 过滤
mfa backup                       # 一键时间戳备份
mfa tui                          # 交互式 TUI：双击复制 / 二维码 / 宠物
```

## 📚 文档

| 文档 | 内容 |
|------|------|
| [docs/install.md](docs/install.md) | 全平台安装：tarball / DEB / RPM / Windows / 源码编译 / 校验和 |
| [docs/usage.md](docs/usage.md) | 命令全表、TUI 快捷键、典型场景、环境变量、文件位置 |
| [docs/faq.md](docs/faq.md) | 排错：PATH / Gatekeeper / 乱码 / 剪贴板 / 天气降级 / 忘锁密码 |
| [SECURITY.md](SECURITY.md) | 安全模型与漏洞上报 |
| [examples/](examples/) | 导入模板（otpauth / JSON） |

## 🔐 安全设计

- **加密**：AES-256-GCM 认证加密 + Argon2id 内存硬派生，每次独立随机 salt / nonce
- **无后门**：忘密码即不可恢复 —— 故启用锁前**强制明文备份**
- **零网络**：OTP 为纯本地计算；天气为唯一可关闭的联网项，离线自动降级
- **隐藏输入**：密码 / 密钥经 `rpassword`，无回显、不进历史；vault 权限 `600`

## 🤝 贡献

欢迎 Issue 与 PR！流程见 [CONTRIBUTING.md](CONTRIBUTING.md)，版本记录见 [CHANGELOG.md](CHANGELOG.md)。

```bash
cargo build && cargo test && cargo clippy   # 本地三连
```

---

## 许可证

基于 [MIT License](LICENSE) 开源，© 2026 Runup01。

## 作者

**Runup01** — [GitHub](https://github.com/Runup01) · bdstravel@126.com

如果 mfacli 对你有帮助，欢迎给仓库点个 ⭐。
