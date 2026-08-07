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
| `--conflict` | `-c` | 已存在时策略：`ask` / `rename` / `skip` / `overwrite` | ask |

```bash
mfa add github --issuer GitHub                 # 推荐：隐藏输入
mfa add aws -s KRSXG5CTMVRXEZLU -i AWS         # 直接传（会留历史，不推荐）
```

> **寻址 `<name|index>`**：所有"指定条目"的命令（`code` / `copy` / `show` / `edit` / `rename` / `remove`）既接受**名称**，也接受 `mfa list` 表格里的**序号**（`INDEX` 列）。
     > - 条目身份 = **(name, issuer)**：同名不同 issuer 是两条独立条目，可共存；
     > - 名称唯一时直接用名称；同名多条时命令会提示改用 index；
     > - 序号是条目的**稳定 ID**：创建/导入时分配（老 vault 首次加载按列表顺序回填 01..N）；删除 / 改名 / 改组都不变号，删掉的号空出、不重用。

### `mfa code <name|index> [-c]`
输出当前验证码（纯 stdout，可管道）。`-c` 同时复制。

### `mfa copy <name|index>`
复制到剪贴板。

### `mfa show <name|index>`
显示密钥 / 算法 / URI + 终端二维码（手机扫描同步）。

### `mfa scan <PATH>... [-n NAME] [-f REGEX]`
解码 QR 图片导入。**支持批量**：多个路径、目录自动递归（png/jpg/jpeg/webp/bmp）。

| 参数 | 说明 |
|------|------|
| `-n/--name` | 自定义名称（仅单文件生效） |
| `-f/--filter` | 简易正则：仅导入 name/issuer 命中的条目。支持 `\|` 或、`*` 通配、`^`/`$` 锚点，忽略大小写 |

```bash
mfa scan qr.png                      # 单张
mfa scan ./qr-dir/                   # 整个目录递归
mfa scan a.png b.jpg -f 'aliyun|tencent'   # 批量 + 过滤
```

重名条目自动 `_2` 递增；结束输出 added/skipped/failed 汇总。

### `mfa list [-l N] [--all] [-g]`
按 issuer→name 排序列出。`-l` 限条数，`--all` 不分页。含 **ADDED** 列（加入日期）；**7 天内**新记录名字后挂 `✦` 标签（TUI 同，底栏另显示完整日期）。

`-g/--group`：**自定义分组小节置顶**（`★ 自定义分组` 区，`▐ 组名 · n ★` 标题）；**其余未加组的条目按正常表格平铺显示**，不再自动生成 issuer 小节，两种条目不会混在一起；自定义组区结尾以暗淡的 `○ 其余条目 ╌╌…` 标明与平铺表格的边界，一眼可见分组到哪里结束。**INDEX 是稳定 ID：入组 / 出组不变号，删除其余条目也不变号**；`list` / `list -g` / TUI 三种视图显示同一套序号。TUI 的 `f` 分组视图同理：自定义组为可折叠的黄色标题（★），其余条目平铺，两区之间有一条不可选中的暗淡 `╌` 分隔行（光标自动跳过）。

**自定义分组**：分组名任意（中文可），条目可随时移入/移出：

```bash
mfa edit github --group 工作      # 单条：加入/改到「工作」组（组名不存在则自动创建）
mfa edit github --group ""       # 移出分组，回到平铺显示
mfa group set 工作 github jms 03  # 批量：一次多条移入（详见下方 mfa group）
```

TUI 里等价操作：`e` 编辑菜单 → `g`。导出（otpauth/JSON）会携带分组信息，导入自动还原。

### `mfa edit <name|index> [OPTIONS]`

| 参数 | 缩写 | 说明 |
|------|------|------|
| `--rename` | `-r` | 新名称 |
| `--secret` | `-s` | 新密钥（不带值则隐藏输入） |
| `--issuer` | `-i` | 新发行方 |
| `--group` | `-g` | 自定义分组名（空字符串 `""` = 移出分组，回到平铺显示） |

```bash
mfa edit github --rename gh
mfa edit github --secret        # 隐藏输入新密钥
mfa edit github --issuer "GitHub Inc"
```

### `mfa rename <name|index> <new>`
重命名（`edit --rename` 快捷方式）。

### `mfa group …`（批量分组管理）
单条调整用 `edit --group`；**批量**调整用这组命令，目标支持名称与序号混写：

```bash
mfa group list                        # 自定义 / 自动分两个区列出
mfa group set 堡垒机 jms 03 tencent-cvm   # 一次性把多条移入组（组不存在自动创建）
mfa group unset jms 03                # 批量移出分组，回到平铺显示
mfa group rename 堡垒机 跳板机          # 重命名自定义组（成员跟随）
```

任一目标无效时**整体中止**，不做部分更改。

### `mfa remove <name|index>...`
删除，支持多个混合：`mfa remove github 02 03`。先全部解析，任一无效则整体中止（防误删）；重复目标自动去重。

`--filter` 模式批量删（如 `mfa remove -f 'aliyun|tencent'`，支持 `|`/`*`/`^`/`$`）：先列命中条目 → **自动备份** → 输 `yes` 才删。

### `mfa export [-o FILE] [-f FMT] [-g GROUP]`

| 格式 | 用途 | 特点 |
|------|------|------|
| `otpauth`（默认） | 通用迁移 | 每行一个 URI，兼容所有 Authenticator |
| `json` | mfacli 间迁移 | 完整保真（含 counter/steam） |
| `encrypted` | 安全备份 | 密码保护，可放云端 |

```bash
mfa export -o tokens.txt                   # otpauth
mfa export -f json -o backup.json          # json
mfa export -f encrypted -o backup.enc      # 加密
mfa export -g 工作 -o work.txt             # 只导出某个分组（三种格式都可组合）
```

`-g/--group`：仅导出属于该组的条目——自定义组名或自动分组键（issuer / 名称前缀），忽略大小写，可用组见 `mfa group list`；组名不存在时错误信息会附上可用组列表。

> 导出的明文文件自动设为 mode 600。妥善保管。

### `mfa import <file> [-s FMT]`
自动检测格式（`encrypted` 除外，须显式 `-s encrypted`）。

检测规则：含 `otpauth-migration://`→google；以 `{`/`[` 开头→json；含 `otpauth://`→otpauth；否则→csv。

```bash
mfa import tokens.txt
mfa import backup.json
mfa import -s encrypted backup.enc
```

**冲突策略**（`add` / `scan` / `import` 统一）：`--conflict` 四选一——
- `ask`（默认）：交互询问 `[y] 覆盖 / [r] 重命名 _2 / [s] 跳过`；非 TTY（脚本/CI）自动回退 `rename`
- `rename`：自动 `_2` 递增，永不覆盖
- `skip`：跳过已存在条目
- `overwrite`：强制覆盖旧条目（保留原加入日期），免确认——批量重导场景用

### `mfa lock [--backup PATH]`
启用应用锁：强制明文备份 → 设密码（两次确认，<8 位额外警告）→ 加密 vault。

### `mfa unlock`
验证密码后解密回明文。

### `mfa backup [-o PATH] [--plain]`
一键备份（时间戳命名，存 `backups/` 目录）。加密库默认导出**加密**备份；`--plain` 强制明文逃生备份。

### `mfa clear`
一键清空全部记录：**先自动明文备份** → 输入 `yes` 确认 → 清空。防手滑。

### `mfa config [OPTIONS]`

| 参数 | 取值 | 说明 |
|------|------|------|
| `--pet` | robot/dino/cat/ghost/dragon | 宠物风格 |
| `--city` | 城市名 | 天气城市 |
| `--show-weather` | on/off | 天气开关 |
| `--show-bazi` | on/off | 黄历开关 |
| `--show-pet` | on/off | 宠物开关 |
| `--keychain` | on/off | 本机免密：vault 密码托管到系统钥匙串 |
| `--qr-style` | half/block | 二维码渲染：half 半高块紧凑（默认）；block 全块，终端字体把 `▀▄` 渲染失真时改用 |
| `--reset` | （无值） | 恢复所有设置为默认（含清除钥匙串托管） |

全局参数 `--no-keychain`：本次命令绕过免密、强制手动输密码（如 `mfa --no-keychain list`）。

```bash
mfa config                      # 查看（含加密状态）
mfa config --pet dino --show-weather false
```

### `mfa tui`
交互式 TUI。

## 本机免密（Keychain / DPAPI）

应用锁启用后每次都要输密码；打开此开关，密码托管到操作系统级机密存储，**第二次起零输入**：

| 系统 | 存储后端 |
|------|------|
| macOS | 登录 Keychain（`/usr/bin/security`） |
| Windows | DPAPI（CurrentUser 作用域，blob 在 `%LOCALAPPDATA%\mfacli\key.dpapi`） |
| Linux | Secret Service（`secret-tool`；无头/无 dbus 环境自动降级回密码输入） |

```bash
mfa config --keychain on     # 开启
mfa list                     # 第一次输密码 → 自动托管，之后免输入
mfa tui                      # 直接进，不再问密码
mfa --no-keychain list       # 单次绕过，强制手输
mfa config --keychain off    # 关闭并清除托管密码
```

TUI 里按 `s` 设置弹窗中也有 **Keychain** 项可切换。

**安全模型（必读）**：
- 防的是「**文件被偷**」：vault.enc 仍是 AES-256-GCM 加密，密码绑定你的 OS 登录会话，文件拷走也打不开。
- 不防「**同机恶意软件**」：同用户进程理论上能读钥匙串（`gh` / `aws cli` 同假设）。公共电脑请保持 off。
- 默认 **opt-in**（off）；改过锁密码导致与钥匙串不匹配时，mfa 自动删除旧托管并回退手动输入。

## TUI 快捷键

| 键 | 功能 | 键 | 功能 |
|---|---|---|---|
| `↑↓`/`jk` | 导航 | `PgUp`/`PgDn` · `Ctrl+U/D` | 翻页（10 行/页） |
| `g`/`Home` · `G`/`End` | 跳首 / 跳尾 | `a` | 添加 |
| `c`/`Enter` | 复制 | `e` | 编辑 |
| 🖱 双击 | 复制（双击分组头 = 折叠/展开） | `r` | 重命名 |
| `v` | 二维码 | `d` | 删除 |
| `f` | 分组视图开关 | `空格` | 折叠/展开当前分组 |
| `s`/`Tab` | 设置 | `q`/`Esc` | 退出 |
| `←→`/`Home`/`End` | 输入弹窗内移动光标 | `Ctrl+V` | 输入弹窗内粘贴剪贴板 |
| `↑↓`/`jk`（二维码层） | 滚动过高的二维码 | `c`（二维码层） | 复制密钥 |
| `Esc`/`v`（二维码层） | 关闭二维码 | `b` | 复制最近备份路径 |

**进阶**：`f` 切换分组视图——**自定义组**以黄色 `▐ 组名 · n ★` 可折叠小节置顶，**其余条目全部平铺**，两区之间有暗淡分隔行标明边界；行内序号是稳定 ID，与 `mfa list` 完全一致；自动分组（issuer / 名称前缀）总览见 `mfa group list`；`空格` 或双击分组头折叠/展开；双击 = 400ms 内连点左键，自动选中行并复制；设置弹窗 `↑↓`+`Enter`，含 **Keychain 免密 / Backup now / Clear all / Reset config**；编辑子菜单 `n`/`i`/`s`/`g`（名称/发行方/密钥/分组），所有输入都在**弹窗**里完成：标题写明正在编辑哪个条目的哪个字段，`←→`/`Home`/`End` 移动光标，`Ctrl+V` 粘贴，`Delete` 删光标后字符，`Enter` 确认、`Esc` 取消；二维码层顶部恒显**名称 + 完整密钥**，过高时 `↑↓` 滚动（提示 `滚动 x-y/N`），`c` 一键复制密钥，`Esc` 关闭；≤5s 变红提醒；7 天内新记录行内挂 `✦`，底栏显示加入日期。

## 典型场景

**脚本 / CI 取码**
```bash
CODE=$(mfa code github)
CODE=$(MFA_PASSWORD="$VAULT_PW" mfa code github)   # 加密 vault
```

**批量扫码入库**
```bash
mfa scan ~/qr/                        # 目录递归，全部 otpauth QR 入库
mfa scan ~/qr/ -f 'aliyun|tencent'    # 只要 name/issuer 命中的
mfa scan a.png -n myname              # 单张可自定义名称
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
| `MFA_NO_KEYCHAIN` | 设为任意值 = 本次绕过钥匙串免密（等同 `--no-keychain`） |

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
