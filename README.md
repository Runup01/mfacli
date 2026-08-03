<p align="center">
  <img src="assets/logo.svg" alt="mfacli" width="520"/>
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

```
  ┌──────────────────────────────────────────────────────────────┐
  │ 2026-08-03 周一 10:30:00 │ Shanghai ☀️ +34°C                  │
  │ 丙午年丙酉月己亥日 满日  │ 宜:祭祀 出行  忌:动土 开仓         │
  └──────────────────────────────────────────────────────────────┘

  ───────────────────────────────────────────────────────────────
  INDEX  NAME          ISSUER        CODE          ⏱
  ───────────────────────────────────────────────────────────────
     01  github        GitHub        482916      18s
     02  aws           AWS           917304       9s
     03  jumpserver    JumpServer    204871      25s
  ───────────────────────────────────────────────────────────────
  tip  mfa copy <name|index>  mfa show <name|index>  mfa tui
```

---

## ✨ 亮点

| | | |
|:---:|:---:|:---:|
| 🔐 **本地优先 · 零云**<br>密钥永不外传；可选应用锁<br>AES-256-GCM + Argon2id | ⌨️ **CLI + TUI 双形态**<br>纯 stdout 可入脚本/CI<br>TUI 双击复制 + 实时倒计时 | 🎯 **协议全覆盖**<br>TOTP · HOTP · Steam Guard<br>SHA1/256/512 · 6/8 位 |
| 📱 **二维码双向**<br>终端渲染 QR 供手机扫描<br>扫描 QR 图片直接导入 | 🔄 **导入导出全格式**<br>otpauth / JSON / CSV / Google 迁移<br>加密备份可上云 | 🐾 **可选趣味装饰**<br>像素宠物 · 天气 · 中国黄历<br>均可关闭，绝不阻塞取码 |

## 🆚 对比

| 能力 | mfacli | Authy | Google Auth | 1Password |
|---|:---:|:---:|:---:|:---:|
| CLI 可脚本化 / 可嵌入 CI | ✅ | ❌ | ❌ | ❌ |
| 交互式 TUI + 鼠标双击复制 | ✅ | ❌ | ❌ | ❌ |
| 完全离线、零云依赖 | ✅ | ❌ | ✅ | ❌ |
| 开源 + 加密存储 | ✅ | 部分 | ❌ | ✅ |
| 单文件、零运行时依赖 | ✅ | ❌ | ❌ | ❌ |
| DEB / RPM 原生包 | ✅ | ❌ | ❌ | ❌ |

## 🚀 快速开始

```bash
# 1) 安装 — macOS Apple Silicon 示例（全平台见 docs/install.md）
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-aarch64-apple-darwin.tar.gz
tar xzf mfacli-aarch64-apple-darwin.tar.gz && chmod +x mfa && sudo mv mfa /usr/local/bin/

# 2) 添加第一个条目（密钥隐藏输入，不进 shell 历史）
mfa add github --issuer GitHub

# 3) 取码 / 进 TUI
mfa code github          # → 482916（纯 stdout，可管道）
mfa copy github          # 复制到剪贴板
mfa remove 2             # 名称或列表序号都能定位条目
mfa tui                  # 交互式 TUI：双击复制 / 二维码 / 宠物
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
