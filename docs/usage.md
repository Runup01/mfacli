# 使用指南 / Usage

## 快速上手

```bash
mfa add github --issuer GitHub     # 添加（密钥隐藏输入，不进历史）
mfa code github                    # 取码 → 482916（纯 stdout）
mfa code github -c                 # 取码并复制
mfa copy github                    # 仅复制
mfa list                           # 列表（含时间/天气/黄历）
mfa show github                    # 详情 + 终端二维码
mfa scan qr.png                    # 扫 QR 图片导入
mfa tui                            # 交互式 TUI
```

## 命令详解

### `mfa`（无参数）
等同 `mfa list`。

### `mfa init [--encrypt]`
初始化 vault（首次 `add` 时自动创建，通常无需手动）。`--encrypt` 直接加密模式。

### `mfa add <name> [OPTIONS]`

| 参数 | 缩写 | 说明 | 默认 |
|------|------|------|------|
| `<name>` | — | 唯一名称（必填） | — |
| `--secret` | `-s` | Base32 密钥（省略则隐藏输入） | 交互 |
| `--issuer` | `-i` | 发行方 | 无 |
| `--algorithm` | `-a` | SHA1 / SHA256 / SHA512 | SHA1 |
| `--digits` | `-d` | 6 / 8 | 6 |
| `--period` | `-p` | 周期秒 | 30 |

```bash
mfa add github --issuer GitHub                 # 推荐：隐藏输入
mfa add aws -s KRSXG5CTMVRXEZLU -i AWS         # 直接传（会留历史，不推荐）
```

> **寻址 `<name|index>`**：所有"指定条目"的命令（`code` / `copy` / `show` / `edit` / `rename` / `remove`）既接受**名称**，也接受 `mfa list` 表格里的**序号**（`INDEX` 列）。
     > - 条目身份 = **(name, issuer)**：同名不同 issuer 是两条独立条目，可共存；
     > - 名称唯一时直接用名称；同名多条时命令会提示改用 index；
     > - 序号按当前排序（issuer→name），唯一且与列表一致。

### `mfa code <name|index> [-c]`
输出当前验证码（纯 stdout，可管道）。`-c` 同时复制。

### `mfa copy <name|index>`
复制到剪贴板。

### `mfa show <name|index>`
显示密钥 / 算法 / URI + 终端二维码（手机扫描同步）。

### `mfa scan <img> [-n NAME]`
解码 QR 图片（PNG/JPG）导入。`-n` 自定义名称。

### `mfa list [-l N] [--all]`
按 issuer→name 排序列出。`-l` 限条数，`--all` 不分页。

### `mfa edit <name|index> [OPTIONS]`

| 参数 | 缩写 | 说明 |
|------|------|------|
| `--rename` | `-r` | 新名称 |
| `--secret` | `-s` | 新密钥（不带值则隐藏输入） |
| `--issuer` | `-i` | 新发行方 |

```bash
mfa edit github --rename gh
mfa edit github --secret        # 隐藏输入新密钥
mfa edit github --issuer "GitHub Inc"
```

### `mfa rename <old|#> <new>`
重命名（`edit --rename` 快捷方式）。

### `mfa remove <name|index>...`
删除，支持多个混合：`mfa remove github 2 3`。先全部解析，任一无效则整体中止（防误删）；重复目标自动去重。

### `mfa export [-o FILE] [-f FMT]`

| 格式 | 用途 | 特点 |
|------|------|------|
| `otpauth`（默认） | 通用迁移 | 每行一个 URI，兼容所有 Authenticator |
| `json` | mfacli 间迁移 | 完整保真（含 counter/steam） |
| `encrypted` | 安全备份 | 密码保护，可放云端 |

```bash
mfa export -o tokens.txt                   # otpauth
mfa export -f json -o backup.json          # json
mfa export -f encrypted -o backup.enc      # 加密
```

> 导出的明文文件自动设为 mode 600。妥善保管。

### `mfa import <file> [-s FMT]`
自动检测格式（`encrypted` 除外，须显式 `-s encrypted`）。

检测规则：含 `otpauth-migration://`→google；以 `{`/`[` 开头→json；含 `otpauth://`→otpauth；否则→csv。

```bash
mfa import tokens.txt
mfa import backup.json
mfa import -s encrypted backup.enc
```

冲突自动重命名（`name_2`），永不覆盖。

### `mfa lock [--backup PATH]`
启用应用锁：强制明文备份 → 设密码（两次确认，<8 位额外警告）→ 加密 vault。

### `mfa unlock`
验证密码后解密回明文。

### `mfa config [OPTIONS]`

| 参数 | 取值 | 说明 |
|------|------|------|
| `--pet` | robot/dino/cat/ghost/dragon | 宠物风格 |
| `--city` | 城市名 | 天气城市 |
| `--show-weather` | true/false | 天气开关 |
| `--show-bazi` | true/false | 黄历开关 |
| `--show-pet` | true/false | 宠物开关 |

```bash
mfa config                      # 查看（含加密状态）
mfa config --pet dino --show-weather false
```

### `mfa tui`
交互式 TUI。

## TUI 快捷键

| 键 | 功能 | 键 | 功能 |
|---|---|---|---|
| `↑↓`/`jk` | 导航 | `a` | 添加 |
| `c`/`Enter` | 复制 | `e` | 编辑 |
| 🖱 双击 | 复制 | `r` | 重命名 |
| `v` | 二维码 | `d` | 删除 |
| `s`/`Tab` | 设置 | `q`/`Esc` | 退出 |

**进阶**：双击 = 400ms 内连点左键，自动选中行并复制；设置弹窗 `↑↓`+`Enter`；编辑子菜单 `n`/`i`/`s`；二维码层 `Esc` 关闭；≤5s 变红提醒。

## 典型场景

**脚本 / CI 取码**
```bash
CODE=$(mfa code github)
CODE=$(MFA_PASSWORD="$VAULT_PW" mfa code github)   # 加密 vault
```

**从其他工具迁移**
```bash
mfa scan google-export.png     # Google Authenticator 截图
mfa import tokens.txt          # otpauth 列表
mfa import backup.json         # JSON
```

**备份与恢复**
```bash
mfa export -f json -o /tmp/backup.json
mfa export -f encrypted -o /tmp/backup.enc
mfa import /tmp/backup.json
mfa import -s encrypted /tmp/backup.enc
```

**应用锁流程**
```bash
mfa lock --backup /secure/usb/plain.json   # 1) 强制备份
# 2) 设密码（两次）；之后每次访问需密码或 MFA_PASSWORD
mfa unlock                                  # 解密回明文
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `MFA_PASSWORD` | 加密 vault 解锁密码（跳过交互，适用脚本/CI）。`lock` 设密码时不读它 |

```bash
MFA_PASSWORD="xxx" mfa code github          # 内联（最安全，不进 history）
export MFA_PASSWORD="xxx"                   # 当前会话
# CI: env: MFA_PASSWORD: ${{ secrets.MFA_VAULT_PASSWORD }}
```

> ⚠️ `export` 形式密码在进程环境可见（`/proc/<pid>/environ`）。多用户主机建议交互输入或 `read -s`。

## 文件位置

| 系统 | 目录 |
|------|------|
| macOS | `~/Library/Application Support/mfa-cli/` |
| Linux | `~/.config/mfa-cli/` |
| Windows | `%APPDATA%\mfa-cli\` |

| 文件 | 说明 | 权限 |
|------|------|------|
| `vault.json` | 明文 vault | 600 |
| `vault.enc` | 加密 vault | 600 |
| `config.json` | 偏好 | 644 |
| `weather_cache.txt` | 天气缓存（30 分钟） | 644 |

迁移机器：复制整个目录；加密 vault 需连同密码。

下一步：[常见问题](faq.md) · [返回主页](../README.md)
