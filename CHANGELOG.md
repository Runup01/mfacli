# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2026-08-03

### Security
- **Argon2id hardened**: KDF params raised to OWASP/RFC 9106 level (64 MiB / 3 passes / p=4); params now stored in a versioned ciphertext header (`MFA1` magic) with tamper caps
- **Legacy vault compatibility**: pre-0.1.5 encrypted vaults (headerless format) still decrypt automatically
- **Key material zeroized**: derived AES key is wiped from memory after use (`zeroize`)
- **Export hardening**: `mfa export -o <file>` and encrypted vault writes now set mode 600 (was default 644)

### Changed
- Documentation restructure: README becomes a product landing page; detailed step-by-step guides moved to `docs/`
- `<name|index>` addressing: `code` / `copy` / `show` / `edit` / `rename` / `remove` accept the list index as well as the name (exact name wins)
- `cargo fmt` applied across the codebase

### Fixed
- `mfa list` column layout no longer collapses with short or single entries (stable min widths: NAME≥16, ISSUER≥12)
- `mfa list` header: `#` column now pads to index width (header/rows align); `mfa list` info header wrapped in a light box (aligned inner `│`, emoji-aware widths); token table keeps open horizontal rules
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
