# 开发沉淀与项目规范 / Dev Notes & Project Memory

> mfacli 从 0 到 v0.1.10 的完整开发记忆（2026-07-29 → 2026-08-04）。
> 记录关键问答决策、踩坑教训与长期规范，供后续维护与贡献者参考。

## 0. 投入概览

- **Token 消耗**：跨会话精确统计不可在代码库内获取（Codex 客户端用量页可查）；
  按会话规模估算为**百万级 token**（含大量截图视觉对齐、编译迭代、CI 修复、多轮 UI 微调）。
- **代码产出**：Rust 约 6,000 行（`src/` 5,958 行），单二进制 `mfa`。
- **文档产出**：README + install / usage / faq / dev-notes 五份，中文为主。
- **发布**：v0.1.4 → v0.1.10 共 7 个版本；CI 双流水线（ci.yml 严格 clippy，release.yml 六平台打包）。

## 1. 关键决策记忆（Q&A 沉淀）

### 产品与体验
- **小而美**：单文件、零运行时依赖、离线优先；增值功能（天气/黄历/宠物）全部后台线程，
  离线自动降级，**绝不阻塞主逻辑响应时间**。
- **双形态**：默认 CLI（脚本友好），`mfa tui` 进交互界面；两边功能对齐（增删改查/二维码/导入导出/设置）。
- **文档语言**：文档中文为主；`--help` 统一英文（避免中英混杂）；运行时提示中文。

### 安全
- 密钥**隐藏输入**，不进 shell history；编辑密钥同样隐藏、不留痕。
- 应用锁（`mfa lock`）启用前**强制明文备份**（逃生绳）+ 风险告知；忘密码无后门。
- Argon2id 采用 OWASP 参数，KDF 参数存密文头（`MFA1` magic），防篡改。
- 钥匙串免密 **opt-in**（`mfa config --keychain on`），旧密码自动清除，`--no-keychain` 单次绕过。
- **事故教训**：真实密钥曾被随仓库推送 → 立即轮换密钥 + 重建仓库历史；
  此后 `examples/` 只允许假数据（`JBSWY3DPEHPK3PXP` 级测试串）。

### 跨平台与打包
- Linux **首推 musl 静态包**：gnu 版 deb/rpm 在老 glibc（CentOS 7 等）报 `GLIBC_2.xx not found`。
- 压缩包**含顶层同名目录**，解压不弄乱当前目录。
- 剪贴板兼容 macOS `pbcopy` / Windows PowerShell / Linux `xclip|xsel|wl-copy`。
- 网络探测**国内 DNS 优先**（223.5.5.5、114.114.114.114），再 fallback 8.8.8.8。
- 天气 IP 无城市名时显示「IP定位」，不显示原始坐标；可 `--city` 精确指定。

### 终端渲染（重灾区，反复迭代）
- **CJK 宽度表**：全角冒号 `：`、全角空格、CJK = 2 列；所有列宽 padding 必须用同一张表，否则歪列。
- **off-by-one 教训**：最长名列的 padding 目标必须 ≥ 最大显示宽度，否则最长行溢出顶歪后续列。
- 颜色要兼顾 **light/dark 两种终端主题**：淡色/高饱和背景都会翻车，最终选中行用加粗而非重背景。
- **二维码**：半高块 `▀▄` 最紧凑，但部分终端字体失真 → 做成可配置 `half`（默认）/ `block`（全块兼容），
  两种都经过「渲染→栅格化→rqrr 反解」端到端验证；otpauth URI 省略默认参数缩小二维码。
- 底栏快捷键**永远完整显示**：状态消息家目录缩写 `~` + 中间截断让位；备份路径按 `b` 一键复制。

### CLI / TUI 交互
- 条目**名称 + 序号双寻址**（序号 01 补零唯一）；删除支持多个 name/index 与 `--filter` 批量。
- 冲突策略统一：`add/scan/import` 共用 `-c ask|rename|skip|overwrite`（默认交互询问）。
- filter 语法 `|` 或、`*` 通配、`^/$` 锚点、忽略大小写，`scan` 与 `remove` 共用。
- 确认输入容忍退格字符（`yes` 带修正不再误取消）；报错要带可操作提示。
- TUI：设置弹窗化（14 项含 QR Style）；双击复制 code；PgUp/PgDn/Ctrl+U/D 翻页、g/G 首尾；
  ADDED 日期列宽屏自动出现；新条目 ✦ 标记（≤7 天）。

## 2. 长期规范

- **版本**：功能/修复变更即 bump；**已推送的 tag 永不复用**（远程 ruleset 不可删 tag，冲突就 bump）。
- **commit 风格**：`release: vX.Y.Z — 摘要` / `feat:` / `docs:` / `ci:`。
- **版本引用同步**：Cargo.toml、Cargo.lock、README、docs/install.md、CHANGELOG 一次改全。
- **新增配置项三同步**：CLI `--x` + TUI 设置项 + docs/usage.md 配置表。
- **CI 严格模式**：`cargo clippy -- -D warnings`；`too_many_arguments` 默认阈值 7，
  参数多了收结构体（`ConfigOpts`），不靠 `#[allow]`。
- **release 流水线**：build/package 多 job 上传 artifact，**单一 release job 发布**，避免并发写竞争。
- 离线开发用 `cargo build/clippy --offline`；不往仓库放任何真实密钥。

## 3. 踩坑清单（别再犯）

| 坑 | 教训 |
|----|------|
| `git push tag` 被拒 | 先看 ruleset 是否限制 tag 创建/删除；tag 已存在就 bump 版本，别反复重推 |
| 字符串按字节切片 | CJK 会 panic（char boundary），用 `get(..n)` 或宽度函数 |
| `print!` 提示不换行 | 必须 `flush()`，否则提示出现在输入之后 |
| Cargo.lock 孤儿包 | 删 dev-deps 后 `cargo update --offline` 清理，否则 example/test 图拉缺失 crate |
| 半高块二维码 | 只在字体正常的终端好看；web 终端常失真 → 永远留 `block` 退路 |
| gnu 构建上老系统 | 老 glibc 必挂；Linux 分发以 musl 静态包为主 |
| 长路径挤掉底栏帮助 | 状态消息要让位于固定 UI，截断 + 缩写 |
