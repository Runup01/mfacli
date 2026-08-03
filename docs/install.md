# 安装指南 / Installation

mfacli 是**静态编译的单文件**，安装 = 把 `mfa` 二进制放进 `PATH`。无运行时、无依赖库。当前版本 **0.1.8**。

## 方式一：下载预编译包（推荐）

从 [Releases](https://github.com/Runup01/mfacli/releases) 下载：

| 平台 | 架构 | 文件 | 安装方式 |
|------|------|------|---------|
| macOS | Apple Silicon (M1–M4) | `mfacli-aarch64-apple-darwin.tar.gz` | 解压 → `chmod +x` → 移入 PATH |
| macOS | Intel | `mfacli-x86_64-apple-darwin.tar.gz` | 同上 |
| Linux（推荐） | x86_64 | `mfacli-x86_64-unknown-linux-musl.tar.gz` | 静态包，全发行版通用：解压 → 移入 PATH |
| Linux（推荐） | ARM64 | `mfacli-aarch64-unknown-linux-musl.tar.gz` | 同上 |
| Windows | x86_64 | `mfacli-x86_64-pc-windows-msvc.zip` | 解压 → `mfa.exe` 放入 PATH |
| Debian/Ubuntu | amd64 | `mfacli_<ver>_amd64.deb` | `sudo dpkg -i` |
| Debian/Ubuntu | arm64 | `mfacli_<ver>_arm64.deb` | `sudo dpkg -i` |
| CentOS/Rocky/Alma | x86_64 | `mfacli-<ver>-1.x86_64.rpm` | `sudo rpm -i` |
| CentOS/Rocky/Alma | aarch64 | `mfacli-<ver>-1.aarch64.rpm` | `sudo rpm -i` |

### macOS（tarball）

```bash
# macOS Apple Silicon（tarball 文件名不含版本号，latest 链接稳定）
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-aarch64-apple-darwin.tar.gz
tar xzf mfacli-aarch64-apple-darwin.tar.gz
chmod +x mfa
sudo mv mfa /usr/local/bin/
mfa --version
```

### Linux（musl 静态包，推荐）

静态链接、零运行时依赖，**任何发行版 / 任何 glibc 版本**（含 CentOS 7、Rocky 8 等老系统）直接可用：

```bash
# x86_64
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-x86_64-unknown-linux-musl.tar.gz
tar xzf mfacli-x86_64-unknown-linux-musl.tar.gz
chmod +x mfa
sudo mv mfa /usr/local/bin/
mfa --version

# ARM64
curl -LO https://github.com/Runup01/mfacli/releases/latest/download/mfacli-aarch64-unknown-linux-musl.tar.gz
tar xzf mfacli-aarch64-unknown-linux-musl.tar.gz
chmod +x mfa
sudo mv mfa /usr/local/bin/
```

> 为什么推荐 musl？deb/rpm 使用 gnu 工具链构建，在 glibc < 2.34 的老系统上会报
> `GLIBC_2.xx not found`；musl 静态包完全自包含，无此问题。
> 需要包管理器统一纳管（升级/卸载/审计）时再选 deb/rpm。

### Debian / Ubuntu（.deb）

```bash
curl -LO https://github.com/Runup01/mfacli/releases/download/v0.1.8/mfacli_0.1.8_amd64.deb
sudo dpkg -i mfacli_0.1.8_amd64.deb
# 或 apt 自动处理依赖：sudo apt install -y ./mfacli_0.1.8_amd64.deb
```

### CentOS / Rocky / AlmaLinux（.rpm）

```bash
curl -LO https://github.com/Runup01/mfacli/releases/download/v0.1.8/mfacli-0.1.8-1.x86_64.rpm
sudo rpm -i mfacli-0.1.8-1.x86_64.rpm
# 升级用：sudo rpm -U mfacli-0.1.8-1.x86_64.rpm
```

### Windows

1. 解压 `mfacli-x86_64-pc-windows-msvc.zip` 得 `mfa.exe`。
2. 放入固定目录，如 `C:\tools\mfacli`。
3. 加入 PATH（PowerShell，无需管理员）：
   ```powershell
   [Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\tools\mfacli", "User")
   ```
   重开终端后 `mfa --version` 生效。

## 方式二：从源码编译

**前置**：[Rust ≥ 1.75](https://rustup.rs/)。

```bash
git clone https://github.com/Runup01/mfacli.git && cd mfacli

# A. cargo install（装到 ~/.cargo/bin，确保在 PATH）
cargo install --path .

# B. 手动构建
cargo build --release
sudo cp target/release/mfa /usr/local/bin/     # Linux/macOS
```

**国内加速**（crates.io 慢时配清华镜像）：

```bash
mkdir -p ~/.cargo && cat >> ~/.cargo/config.toml << 'CFG'
[source.crates-io]
replace-with = 'tuna'

[source.tuna]
registry = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/"
CFG
```

## 校验完整性

每个 Release 资源在 GitHub 页面附带 `sha256`：

```bash
shasum -a 256 mfacli-aarch64-apple-darwin.tar.gz   # macOS
sha256sum <file>                                   # Linux
certutil -hashfile <file> SHA256                   # Windows
```

## 平台细节

### macOS
- **执行权限**：`Permission denied` → `chmod +x mfa`
- **Gatekeeper**（"无法验证开发者"）：`xattr -d com.apple.quarantine /usr/local/bin/mfa`，或右键→打开→确认
- **装错架构**：`exec format error` → 用 `uname -m` 核对（`arm64` 选 aarch64，`x86_64` 选 x86_64）

### Linux
- **架构核对**：`uname -m`
- **SELinux/AppArmor**：仅读写用户配置目录与调用剪贴板程序，默认策略无需放行
- **无 root**：tarball 放 `~/.local/bin`（在 PATH 中）即可

### Windows
- **Defender 误报**：小众 Rust 二进制偶发拦截，对该文件"允许"或加白名单
- **终端**：推荐 Windows Terminal（UTF-8/Unicode 最佳）；老 `cmd` 中文乱码见 [FAQ](faq.md)

## 验证安装

```bash
mfa --version   # → mfa 0.1.8
mfa --help      # 全部子命令
```

下一步：[使用指南](usage.md)。
