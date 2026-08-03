# Security Policy

## Supported Versions

| Version | Supported |
|---------|:---------:|
| 0.1.x   | ✅        |

## Reporting a Vulnerability

If you discover a security vulnerability, please **do not** open a public issue.
Instead, email **bdstravel@126.com** with:

- A description of the vulnerability
- Steps to reproduce
- Affected versions
- Any suggested fix (optional)

We aim to respond within 48 hours and will coordinate a fix + disclosure timeline.

## Security Design

### Encryption

- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key derivation**: Argon2id, 64 MiB / 3 passes / parallelism 4 (OWASP & RFC 9106 aligned); params stored in a versioned `MFA1` ciphertext header; legacy headerless vaults auto-detected and still decryptable
- **Salt**: 16 bytes random per encryption
- **Nonce**: 12 bytes random per encryption
- **Key material**: derived key zeroized from memory immediately after cipher construction
- **No backdoors**: There is no password recovery mechanism. If you forget your lock password, the encrypted vault is unrecoverable. This is by design.

### Storage

- Default vault location: `~/.config/mfa-cli/vault.json` (plain, mode 600)
- Encrypted vault: `~/.config/mfa-cli/vault.enc` (AES-256-GCM ciphertext)
- Config: `~/.config/mfa-cli/config.json` (non-sensitive UI preferences)

### App Lock

- `mfa lock` enables password protection for all vault access
- **Mandatory backup**: A plain-text backup is written *before* encryption is applied
- **Double-confirm**: Password must be entered twice to prevent typos
- **Minimum length warning**: Passwords under 8 characters trigger an extra confirmation
- **No password in arguments**: Passwords are never accepted as CLI flags (would leak to `ps`/history)

### Secret Input

- `mfa add` without `--secret` prompts with hidden input (via `rpassword`)
- `mfa edit --secret` without a value prompts with hidden input
- TUI edit mode uses raw terminal input (no echo, no history)

### What mfacli Does NOT Do

- **No network access** for core functionality (OTP generation is pure local computation)
- **No telemetry** or analytics of any kind
- **No cloud sync** (by design — your secrets never leave your machine unless you explicitly export them)
- **No password recovery** (AES-GCM has no backdoor; this is a feature, not a bug)

## Incident Response: Accidental Secret Exposure

If you accidentally commit or expose OTP secrets (e.g. via `git add -A`):

1. **Rotate immediately**: Go to each affected service and reset/regenerate the MFA secret
2. **Clean git history**: `git filter-branch` or `git rebase -i` to remove the commit, then force-push
3. **Delete remote refs**: Remove any tags/branches that reference the dirty commit
4. **Consider the data leaked**: Even after git cleanup, assume the secrets were captured by crawlers/mirrors. Rotation is the only true remediation.
5. **Harden .gitignore**: Add patterns to prevent recurrence (e.g. `/*.txt`, `/*.json`)

### Prevention

- Always export to `/tmp/` or outside the repo: `mfa export -o /tmp/tokens.txt`
- Use `mfa add` without `--secret` (hidden prompt, no history)
- Use file-based import (`mfa import auth.txt`) instead of inline secrets
- Review `git status` before `git add -A`
- The `.gitignore` includes `/*.txt`, `/*.json`, `/*.enc` as a safety net for root-level dumps
