# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> ⚠️ Versions below 1.0.0 may have breaking algorithm changes.
> A breaking algorithm change will always bump the MAJOR version and
> update `version` in ALGORITHM.md.

---

## [Unreleased]

## [0.2.1] — 2026-06-19

### Added
- First-run setup wizard built into `hdpassw-gui`: generate a new seed phrase
  or restore an existing one, complete a 6-word recall quiz to catch
  transcription mistakes, and save the encrypted vault — entirely from the
  GUI, with no CLI step required beforehand
- BIP39 word autocomplete (suggestion chips) in the GUI's seed-verification
  quiz and seed-restore screens
- `hdpassw init`: a friendlier first-run entry point that asks whether to
  create a new seed phrase or restore an existing one, then hands off to
  `seed new` / `seed restore`

### Fixed
- Centering of buttons and the seed-word grid in the GUI's setup wizard

## [0.2.0] — 2026-06-13

### Added
- Optional `hdpassw-gui` binary: a pure-Rust (egui/eframe) desktop GUI for
  unlocking the seed vault, generating/copying passwords, and adding sites —
  built and installed via the `gui` Cargo feature (`make gui`, `./build.sh --gui`)
- Desktop launcher (`.desktop` entry + icon) installed alongside the GUI
  via `make install-gui` / `./install.sh --gui`
- `src/manager.rs`: shared password-generation and site-upsert logic used by
  both the CLI and the GUI, so they can never diverge

### Changed
- Reorganized the crate into a library (`hdpassw`) plus two binaries
  (`hdpassw`, `hdpassw-gui`), both depending on the shared library for
  session, store, clipboard, and crypto code
- Vault password is now held in a `Zeroizing<String>` in the GUI, matching
  the CLI's handling of secret material

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
