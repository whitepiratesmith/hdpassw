# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> ⚠️ Versions below 1.0.0 may have breaking algorithm changes.
> A breaking algorithm change will always bump the MAJOR version and
> update `version` in ALGORITHM.md.

---

## [Unreleased]

## [0.1.0] — 2026-05-24

### Added
- Initial release
- BIP39 24-word mnemonic generation and validation (`hdpassw seed new/check`)
- Deterministic password derivation:
  - scrypt (N=2^17, r=8, p=1) for seed → master key (memory-hard; 128 MiB per attempt)
  - BLAKE3 key-derivation mode for master key → site key
  - BLAKE3 XOF for site key → password charset encoding
- Argon2id + ChaCha20-Poly1305 for optional encrypted vault (`~/.config/hdpassw/seed.vault`)
- Five character sets: `alpha`, `alphanumeric`, `full`, `pin`, `hex`
- Site metadata store at `~/.config/hdpassw/sites.toml` (no secrets stored)
- Clipboard copy with configurable auto-clear (`hdpassw gen`)
- Site record management (`hdpassw add`, `hdpassw ls`, `hdpassw rm`)
- Metadata export (`hdpassw export`)
- Counter-based password rotation
- Verifier field for seed-only counter recovery (`hdpassw recover`)
- Shell completions for bash, zsh, fish
- Full cryptographic specification in `ALGORITHM.md`
- Threat model and security policy in `SECURITY.md`
- GitHub Actions CI: build, test, clippy, fmt, `cargo audit`
