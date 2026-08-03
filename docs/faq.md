# 常见问题 / FAQ

> 安装见 [install.md](install.md)，使用见 [usage.md](usage.md)。本页覆盖排错与各种"为什么"。

## 安装与运行

**Q：`mfa: command not found`？**
二进制不在 `PATH`。用 `which mfa`（Windows：`where mfa`）确认位置；把所在目录加入 `PATH` 后**重开终端**。tarball 安装默认放在 `/usr/local/bin` 或 `~/.local/bin`。

**Q：macOS 提示"无法打开 / 无法验证开发者"？**
下载文件带隔离属性，移除即可：

```bash
xattr -d com.apple.quarantine /usr/local/bin/mfa
# 或首次右键 → 打开 → 在弹窗中确认
```

**Q：`exec format error` / 运行不了？**
装错架构。`uname -m` 核对：`arm64` 选 aarch64 包，`x86_64` 选 x86_64 包。

**Q：如何升级 / 卸载？**

```bash
# 升级
sudo dpkg -i ./mfacli_<ver>_amd64.deb      # deb 直接覆盖
sudo rpm -U ./mfacli-<ver>-1.x86_64.rpm     # rpm 用 -U
# tarball：重新解压覆盖 /usr/local/bin/mfa 即可

# 卸载
sudo dpkg -r mfacli / sudo rpm -e mfacli    # 或直接删除二进制
# 数据在 ~/.config/mfa-cli（macOS: ~/Library/Application Support/mfa-cli），按需删除
```

## 显示与 locale

**Q：Linux 中文乱码？**
TUI 与列表依赖 UTF-8：

```bash
export LANG=zh_CN.UTF-8 LC_ALL=zh_CN.UTF-8   # 或 en_US.UTF-8
# 持久化写入 ~/.bashrc / ~/.zshrc；Debian 系可 sudo locale-gen zh_CN.UTF-8
```

**Q：Windows 中文乱码？**
推荐 **Windows Terminal**（自带 UTF-8）；老 `cmd` 先执行 `chcp 65001`。

**Q：TUI 二维码花屏？**
终端字体不支持半块字符（▀▄）。换等宽 Unicode 字体（如 JetBrains Mono / Sarasa），或改用 `mfa show <name>` 查看。

## 剪贴板

**Q：复制没反应？**
macOS（`pbcopy`）/ Windows（`clip`）内置无需安装；Linux 需要：

| 环境 | 依赖 | 安装 |
|------|------|------|
| X11 | `xclip` 或 `xsel` | `sudo apt/yum/dnf install -y xclip` |
| Wayland | `wl-clipboard` | `sudo apt/dnf install -y wl-clipboard` |
| Arch | 两者 | `sudo pacman -S xclip wl-clipboard` |

mfacli 按 `xclip → xsel → wl-copy` 顺序自动尝试。**纯 SSH 无图形会话**剪贴板不可用属预期行为，用 `mfa code <name>` 取码手动输入。

## 天气与黄历

**Q：天气不显示？**
天气取自 **wttr.in**，国内部分地区不可达时**静默降级**（不显示、不报错、不阻塞取码）。可用 `mfa config --show-weather false` 彻底关闭，避免启动探测等待。

**Q：天气会泄露隐私吗？**
仅一次 HTTP GET 到 wttr.in（城市名或自动 IP 定位），无其他上报；30 分钟本地缓存。核心 OTP 计算**完全离线**。

**Q：黄历 / 八字要联网吗？**
不要。天干地支、建除十二神、宜忌均为纯本地离线计算。

## 安全与加密

**Q：数据存在哪？**
用户配置目录（macOS `~/Library/Application Support/mfa-cli/`、Linux `~/.config/mfa-cli/`、Windows `%APPDATA%\mfa-cli\`）。`vault.json` 明文 / `vault.enc` 加密，权限均 `600`。

**Q：忘了应用锁密码怎么办？**
加密 vault **不可恢复**（AES-256-GCM 无后门）。用 `mfa lock` 时强制生成的明文备份恢复：`mfa import <备份.json>`。这也是启用锁前必须先备份的原因。

**Q：`MFA_PASSWORD` 在脚本里安全吗？**
`MFA_PASSWORD="x" mfa ...` 内联形式最干净（密码只存在于该子进程）。`export` 形式密码在进程环境可见（`/proc/<pid>/environ`、`ps eww`），多用户主机请改用交互输入。切勿把明文写进 shell history。

**Q：加密备份能放云端吗？**
可以。`mfa export -f encrypted` 产出 AES-256-GCM + Argon2id 加密文件，强度取决于你的密码长度（建议 ≥ 12 位）。

## 导入与导出

**Q：导入支持哪些格式？**
自动检测：`otpauth://` 列表、JSON、CSV、Google Authenticator 迁移文件。**`encrypted` 不自动检测**，须显式 `mfa import -s encrypted <file>`（防误把加密文件当文本解析）。

**Q：导入时名称重复会怎样？**
自动重命名（如 `github_2`），**永不覆盖**已有条目；结束会汇报导入 / 跳过条数。

**Q：导出三种格式怎么选？**
- `otpauth`（默认）：通用迁移，任何认证器 App 可扫 / 可导入
- `json`：完整保真备份（明文，妥善保管）
- `encrypted`：密码保护备份，可放心放云端

导出格式即导入模板，闭环互通。模板见 [`examples/`](../examples/)。

## 其他

**Q：会遥测 / 联网上报吗？**
不会。零遥测、零云同步；天气是唯一可选联网项，可关闭。

**Q：发现漏洞怎么报？**
邮件 **bdstravel@126.com**，勿公开提 Issue。详见 [SECURITY.md](../SECURITY.md)。

---

[安装指南](install.md) · [使用指南](usage.md) · [返回主页](../README.md)
