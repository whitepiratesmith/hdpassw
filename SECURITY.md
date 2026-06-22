# Security Policy

## Threat Model

hdpassw is a **stateless, deterministic** password manager.

### What it protects

| Asset | Protection |
|---|---|
| Master key | Never stored; derived on demand; held in locked (`mlock`) memory; zeroed on drop |
| Passwords | Never stored; derived on demand; clipboard auto-cleared after timeout |
| Seed phrase | Never stored by the program; user is responsible for offline backup |
| Metadata file | Contains no secrets; safe to store in plaintext, git, cloud |

### Assumptions

- The seed phrase is stored securely offline (paper, metal backup)
- The device running hdpassw is not compromised at runtime
- The OS clipboard is trusted (hdpassw cannot protect against clipboard sniffers)

### Out of scope

- Keyloggers or screen-capture malware on the host
- Side-channel attacks on BLAKE3/scrypt in software
- Clipboard sniffers (use `--reveal` only in a trusted terminal)
- Physical access to the device while the process is running

### Network access

`hdpassw` is otherwise fully offline. The only exception is
`hdpassw pwned` (CLI) and the "Test if pwned" button (GUI), which query the
[Have I Been Pwned](https://haveibeenpwned.com/API/v3#PwnedPasswords)
"Pwned Passwords" range API over HTTPS. This uses k-anonymity: only the
first 5 hex characters of the derived password's SHA-1 hash are sent —
never the password itself, the full hash, the site name, or the seed
phrase. No API key is configured or required. Skip these commands entirely
if you do not want hdpassw to make any network requests.

---

## Cryptographic Primitives

| Purpose | Primitive | Standard |
|---|---|---|
| Mnemonic → seed | PBKDF2-HMAC-SHA512, 2048 rounds | BIP39 |
| Seed → master key | scrypt N=2^17, r=8, p=1 | RFC 7914 |
| Master key → site key | BLAKE3 key-derivation mode | BLAKE3 spec |
| Site key → password | BLAKE3 XOF + modular reduction | BLAKE3 spec |
| Vault encryption key | Argon2id | RFC 9106 |
| Vault cipher | ChaCha20-Poly1305 | RFC 8439 |
| Random generation | OS CSPRNG via `getrandom` | — |
| Pwned-password lookup key | SHA-1 (k-anonymity prefix only, per HIBP API) | — |

No custom cryptographic constructions are used. All primitives are
implemented by audited Rust crates (`scrypt`, `blake3`, `argon2`,
`chacha20poly1305`, `bip39`, `rand`).

See [ALGORITHM.md](ALGORITHM.md) for the full specification.

---

## Attack Scenarios

### Compromised metadata file

The metadata file (`sites.toml`) contains site names, usernames, counters,
and 4-char verifiers. **No passwords or key material.**

An attacker who obtains the metadata file learns:
- Which sites you have accounts on
- Your usernames

They cannot derive any password without the seed phrase.

### Compromised single password

Passwords are derived via BLAKE3 KDF with site-specific key material.
Compromising one password (e.g. via a site breach) reveals nothing about
other passwords or the master key — BLAKE3's KDF mode provides strong
one-way domain separation.

Rotate the compromised site's password by incrementing its counter and
updating the metadata file.

### Brute-force of seed phrase

A 24-word BIP39 mnemonic has 256 bits of entropy. Brute force is
computationally infeasible.

A 12-word mnemonic has 128 bits of entropy, also infeasible, but 24 words
is the default.

### Loss of seed phrase

Without the seed phrase, all passwords are permanently unrecoverable.
Back it up offline before use.

### Loss of metadata file

The seed phrase alone can regenerate all passwords at counter=1. Passwords
that were rotated (counter > 1) require knowledge of the current counter.

The verifier field (stored in the metadata file) allows recovering the
correct counter by scanning `n = 1, 2, 3, …` until the verifier matches.
See `hdpassw recover --help`.

---

## Reporting Vulnerabilities

Please report security vulnerabilities by email to **jakobhusu@jakobhusu.com**.

Do not open a public GitHub issue for security vulnerabilities.

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (optional)

Expected response time: 72 hours.
