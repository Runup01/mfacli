# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.15] - 2026-08-07

### Added
- TUI 输入弹窗：添加 / 重命名 / 编辑 / 导入 / 导出统一改为独立黄框弹窗——标题写明正在编辑的条目（`编辑: 名称 (发行方)`）与当前字段，字段与输入区之间有分隔线，一眼看清写在哪里
- 输入光标：`←`/`→`/`Home`/`End` 移动光标，`Delete` 删除光标后字符，Backspace 支持中间删除；长内容自动横向滚动，光标始终可见
- `Ctrl+V` 粘贴：任意输入框可把系统剪贴板内容粘贴到光标处（自动过滤控制字符），成功 / 失败都有提示
- 二维码弹窗可滚动：二维码高于可用空间时 `↑`/`↓`（或 `j`/`k`）逐行滚动，底部显示 `滚动 x-y/N` 窗口提示
- 二维码弹窗内按 `c` 一键复制密钥（成功提示附带「请勿外泄」）

### Changed
- 二维码弹窗顶部恒显条目名称与完整密钥（过长自动换行），二维码居中；长名称 / 长密钥不再被挤出视野
- TUI 输入态底栏统一为 `✎ 输入中 …（见弹窗）`，原先四组底栏输入行合并进弹窗

### Fixed
- 修复二维码过高时密钥被顶出弹窗、长条目名称被截断看不全的问题

## [0.1.14] - 2026-08-07

### Added
- `mfa export --group <g>` (`-g`): export only the entries of one group (custom group name or auto group key, case-insensitive) in any format (otpauth / json / encrypted); an unknown group prints the available group list

## [0.1.13] - 2026-08-04

### Changed
- INDEX is now a stable per-entry id: assigned once at creation (legacy vaults get 01..N backfilled in list order on first load) and never renumbered — removing entry 01 no longer shifts 02→01, so numbers read from the screen stay valid across consecutive commands; removed ids leave gaps instead of being reused

### Added
- `mfa list` prints a dim note when legacy entries (imported before v0.1.7) have no ADDED date

## [0.1.12] - 2026-08-04

### Fixed
- INDEX numbers are now stable and identical everywhere (`mfa list`, `mfa list --group`, TUI, and all `<name|index>` commands): they are positions in the flat issuer→name order and no longer shift when entries join/leave custom groups — previously a number read from a stale screen could hit the wrong entry (e.g. `mfa group set X 26` right after creating a group); the TUI also used vault insertion order before and could disagree with the CLI
- `mfa list --group` with no custom groups no longer prints a stray `○ 其余条目` separator above the table

## [0.1.11] - 2026-08-04

### Added
- Custom groups: `mfa edit <name|index> --group <group>` assigns any entry to any (auto-created) group, `--group ""` removes it; TUI equivalent is the edit menu `g`. Groups survive export/import (otpauth `&group=` param + JSON)
- Batch group management: `mfa group list / set <group> <name|index>... / unset <name|index>... / rename <old> <new>` — move any mix of names and indexes into a (auto-created) group in one shot; aborts entirely on any invalid target
- Group / fold view for long vaults: `mfa list --group` / TUI `f` show your custom groups as ★ sections on top while every other entry stays in the normal flat table; `space` (or double-click on a header) folds/expands a section; `mfa group list` still lists custom + auto groups as an overview

### Changed
- Group headers redesigned: colored accent bar (`▐`, yellow=custom / cyan=auto) + dotted leader, replacing the plain `──` line
- Custom-group sections now end with an explicit dim boundary (`○ 其余条目 ╌╌…` in the CLI, a non-selectable divider row in the TUI) so it is always obvious where your groups end and the flat list begins

### Fixed
- Piping into `head` etc. (`mfa list | head -3`) no longer panics with "Broken pipe" — SIGPIPE default behavior restored, exits quietly like standard Unix tools

## [0.1.10] - 2026-08-03

### Changed
- Releases now also ship version-less alias packages (`mfacli_amd64.deb`, `mfacli_x86_64.rpm`, …) so `releases/latest/download/` links in docs always install the newest version
- Release tarballs / zips now contain a single top-level directory (e.g. `mfacli-x86_64-unknown-linux-musl/`) instead of loose files, so extracting never clutters your current directory
- Docs: Linux quick start recommends the musl static package (works on any distro / old glibc, e.g. CentOS 7)

### Fixed
- Weather header shows "IP定位" instead of raw coordinates (e.g. 34.773200) when the IP has no city name; set an exact city via `mfa config --city <name>`
- TUI list columns no longer shift by one on the longest-name row (name column padding off-by-one)
- `scan` conflict prompt now tells "vault 中已存在" apart from "本批次重复" (duplicate QR images inside one batch no longer look like ghost entries)
- QR display is now configurable: `half` (default, compact half-blocks) and `block` (full-block '█' only, for terminals whose fonts distort half-blocks) — switch via `mfa config --qr-style half|block` or TUI Settings → QR Style; both verified decodable end-to-end
- otpauth URIs now omit spec-default params (SHA1/6/30), shrinking the QR payload and thus the on-screen QR size

## [0.1.8] - 2026-08-03

### Added
- Unified conflict policy for `add` / `scan` / `import`: `--conflict ask|rename|skip|overwrite` (interactive ask by default, force-overwrite for batch re-imports)
- `mfa remove --filter <pattern>` bulk delete by name/issuer pattern (auto-backup + typed yes confirmation)

### Fixed
- Small / low-resolution QR images now decode via 2x–4x upscale retry (Nearest + Triangle); previously failed with "no QR detected"
- Confirmation prompts ignore stray backspace/delete characters (`yes` typed with corrections no longer mis-cancels)
- TUI: long status messages (e.g. backup paths) are home-abbreviated + middle-truncated so the shortcut bar never gets pushed off-screen; `b` copies the last backup path to the clipboard
- TUI: list shows the ADDED date column on wide terminals (matches CLI)

## [0.1.7] - 2026-08-03

### Added
- `mfa backup` one-click timestamped backup (encrypted vault → encrypted backup; `--plain` escape hatch)
- `mfa clear` wipe-all with mandatory auto-backup + typed yes confirmation (CLI & TUI)
- `mfa config --reset` restore defaults (clears keychain store); TUI settings gains Backup / Clear all / Reset config
- `mfa scan` batch mode: multiple paths + recursive directory scan; `--filter` pattern (`|`, `*`, `^`, `$`, case-insensitive)
- Entries carry `created_at`; `list` shows ADDED column; entries <7 days get a ✦ marker in CLI & TUI

## [0.1.6] - 2026-08-03

### Added
- **OS keychain passwordless (opt-in)**: `mfa config --keychain on` stores the vault password in macOS Keychain / Windows DPAPI / Linux Secret Service; zero-input after first unlock; `--no-keychain` per-run bypass; TUI settings toggle; stale stored password auto-deleted on mismatch

### Fixed
- Interactive yes/no prompts now flush stdout immediately (prompt text no longer prints after the error line)
- `mfa unlock` uses a clear two-step flow: verify password first, then confirm y/N

## [0.1.5] - 2026-08-03

### Security
- **Argon2id hardened**: KDF params raised to OWASP/RFC 9106 level (64 MiB / 3 passes / p=4); params now stored in a versioned ciphertext header (`MFA1` magic) with tamper caps
- **Legacy vault compatibility**: pre-0.1.5 encrypted vaults (headerless format) still decrypt automatically
- **Key material zeroized**: derived AES key is wiped from memory after use (`zeroize`)
- **Export hardening**: `mfa export -o <file>` and encrypted vault writes now set mode 600 (was default 644)

### Changed
- Documentation restructure: README becomes a product landing page; detailed step-by-step guides moved to `docs/`
- `<name|index>` addressing: `code` / `copy` / `show` / `edit` / `rename` / `remove` accept the list index as well as the name (exact name wins)
- `mfa remove` accepts multiple targets (names and/or indexes, deduped); invalid target aborts the whole batch
- Linux binaries are now fully static (musl): run on any distro incl. CentOS 7, no glibc version errors
- `mfa import` rename messages now distinguish "already exists in vault" from "duplicate in import file"
- Entry identity is now (name, issuer): same name with different issuers coexists; ambiguous name addressing asks for the index; TUI add/rename/edit/import enforce the same uniqueness
- Base32 secrets are now normalized (uppercased, separators stripped) on import and tolerated at generation time; legacy lowercase imports generate codes again; unrecoverable codes render red instead of green
- `cargo fmt` applied across the codebase

### Fixed
- `mfa list` column layout no longer collapses with short or single entries (stable min widths: NAME≥16, ISSUER≥12)
- `mfa list` header: `#` renamed to INDEX (width adapts to digit count) (header/rows align); `mfa list` info header wrapped in a light box (aligned inner `│`, emoji-aware widths); token table keeps open horizontal rules
- TUI: same stable min column widths; selection no longer shifts columns (fixed 2-cell `▸` lead + whole-row bold)

### Added
- Dependabot config for continuous dependency (SCA) scanning
- Regression tests: legacy-format decrypt, magic-header presence

## [0.1.3] - 2026-08-02

Initial public release.

### Core OTP
- **TOTP** (RFC 6238), **HOTP** (RFC 4226), **Steam Guard**
- Algorithms SHA1 / SHA256 / SHA512; 6 / 8 digits; configurable period

### CLI
- `add` / `code` / `copy` / `list` / `show` / `scan` / `edit` / `rename` / `remove`
- `code` prints pure stdout (pipe / script / CI friendly); `-c` also copies
- Hidden secret input via `rpassword` (no shell history) on `add` and `edit --secret`
- Friendly, actionable error messages for both clap parsing and business errors
- CJK-safe, display-width-aware column alignment; entries sorted by issuer then name

### Interactive TUI
- Real-time code refresh with countdown progress bar
- **Mouse double-click to copy** (400ms detection, row-mapped)
- Full keyboard management: add / edit / rename / delete / QR / settings
- Pixel pet companions (robot / dino / cat / ghost / dragon) with mood animation
- Weather (auto IP, 30-min cache, offline-safe) and Chinese almanac (天干地支 / 建除 / 宜忌)
- Copy feedback: `✓ 已复制 <name> → <code>  可粘贴`

### QR codes
- `show` renders a terminal QR for phone sync; `scan` decodes a QR image to import

### Import / Export
- Import auto-detects `otpauth://`, JSON, CSV, Google Authenticator migration
- Encrypted import is explicit (`-s encrypted`)
- Export formats: `otpauth` (default, universal), `json` (full fidelity), `encrypted`
- Rename-on-collision; full CJK / special-character support

### Security & storage
- Optional **App Lock**: AES-256-GCM + Argon2id, mandatory plain-text backup before enabling
- Vault file mode 600; `MFA_PASSWORD` env var for non-interactive / CI use
- `.gitignore` hardened against accidental secret commits

### Packaging & distribution
- Single static binary, zero runtime dependencies
- Release builds for macOS (ARM/Intel), Linux (x86_64/ARM64), Windows (x86_64)
- Native **DEB** (amd64/arm64) and **RPM** (x86_64/aarch64) packages
- Cross-platform clipboard (pbcopy / clip / xclip / xsel / wl-copy)
- GitHub Actions CI (test + clippy on 3 OS) and automated release workflow
