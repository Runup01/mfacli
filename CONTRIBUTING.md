# Contributing to mfacli

Thank you for your interest in contributing! This document provides guidelines for contributing to mfacli.

## Development Setup

```bash
git clone https://github.com/Runup01/mfacli.git
cd mfacli
cargo build
cargo test
cargo clippy
```

## Code Style

- Follow standard Rust conventions (`rustfmt` default config)
- Run `cargo clippy` before submitting — fix or explicitly `#[allow]` with a comment
- Keep functions focused; prefer small, testable units
- No `unwrap()` in production paths — use `?` or explicit error handling
- All user-facing strings should be in English (CLI output); comments may be bilingual

## Testing

- Unit tests go in the same file as the code (`#[cfg(test)] mod tests { ... }`)
- Integration tests go in `tests/`
- Run `cargo test` before submitting a PR
- New features must include at least one test covering the happy path

## Security

- **Never commit secrets, vault files, or OTP keys** — see [SECURITY.md](SECURITY.md)
- If you accidentally commit sensitive data, follow the incident response in SECURITY.md
- Password prompts must use `rpassword` (hidden input, no history)
- All encryption uses AES-256-GCM + Argon2id — do not weaken parameters

## Pull Request Process

1. Fork the repo and create a feature branch (`feat/...`, `fix/...`, `docs/...`)
2. Make your changes with clear commit messages
3. Ensure `cargo build`, `cargo test`, and `cargo clippy` all pass
4. Update documentation if your change affects user-facing behavior
5. Open a PR against `main` with a description of what and why
6. Wait for review — we aim to respond within 48 hours

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add HOTP counter display
fix: CJK-safe truncation in list view
docs: update import/export section
ci: add clippy job to release workflow
```

## Questions?

Open a [GitHub Issue](https://github.com/Runup01/mfacli/issues) or start a [Discussion](https://github.com/Runup01/mfacli/discussions).
