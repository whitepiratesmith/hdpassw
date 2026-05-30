# hdpassw — Cryptographic Algorithm Specification

Version: 1  
Status: Stable

This document defines the exact algorithm used to derive passwords from a
seed phrase. Any correct implementation must produce identical output for
the same inputs.

---

## Overview

```
Mnemonic (12–24 BIP39 words)
    │
    │  + Optional passphrase
    ▼
BIP39 Seed  (64 bytes)          PBKDF2-HMAC-SHA512, 2048 rounds
    │
    ▼
Master Key  (32 bytes)          scrypt  N=2^17, r=8, p=1, salt="hdpassw-v1"
    │
    │  + site name + username + counter
    ▼
Site Key    (32 bytes)          BLAKE3 key-derivation mode
    │
    ▼
Password    (N chars)           BLAKE3 XOF + charset encoding
```

---

## Step 1 — Mnemonic to Seed (BIP39)

Standard BIP39 seed derivation, unchanged:

```
seed = PBKDF2-HMAC-SHA512(
    password  = normalize_nfkd(mnemonic_sentence),
    salt      = "mnemonic" || normalize_nfkd(passphrase),
    rounds    = 2048,
    dklen     = 64
)
```

- `mnemonic_sentence`: space-separated BIP39 words, NFKD-normalized
- `passphrase`: user-supplied second factor; empty string `""` is valid
- Output: 64-byte seed

Test vector (from BIP39 spec):

| Field      | Value |
|---|---|
| Mnemonic   | `abandon` ×11 + `about` |
| Passphrase | `TREZOR` |
| Seed[0..2] | `0xc5 0x52` |

---

## Step 2 — Seed to Master Key (scrypt)

```
master_key = scrypt(
    password = seed[0..32],
    salt     = b"hdpassw-v1",
    N        = 131072,          // 2^17
    r        = 8,
    p        = 1,
    dklen    = 32
)
```

- Only the first 32 bytes of the seed are used; the upper 32 are reserved for
  future key types.
- The fixed application salt `"hdpassw-v1"` domain-separates this derivation
  from any other tool (e.g. a cryptocurrency wallet) that might process the
  same BIP39 seed bytes.
- scrypt at these parameters requires 128 MiB of RAM per attempt, making
  GPU/ASIC brute-force expensive even when a weak passphrase is used.

---

## Step 3 — Master Key to Site Key (BLAKE3 KDF)

```
context  = "hdpassw 2026-05-24 site-key"
ikm      = master_key || "site:" || site_name || "|user:" || username || "|n:" || counter
site_key = BLAKE3-KDF(context, ikm)
```

Where:
- `context`: hardcoded, globally unique string — provides domain separation
- `master_key`: 32-byte output of Step 2
- `site_name`: UTF-8 string, e.g. `"github.com"`
- `username`: UTF-8 string, e.g. `"jakob"`
- `counter`: decimal ASCII string, e.g. `"1"`, `"2"`, …
- `||`: byte concatenation (no length-prefix; fields are delimited by `|`)
- Output: 32-byte site key

Example `ikm` (after the master key bytes):
`site:github.com|user:jakob|n:1`

BLAKE3 key-derivation mode is instantiated as:
`blake3::Hasher::new_derive_key(context).update(ikm).finalize()`

---

## Step 4 — Site Key to Password (BLAKE3 XOF + charset encoding)

```
context  = "hdpassw 2026-05-24 encode"
reader   = BLAKE3-KDF-XOF(context, site_key)
expanded = reader.read(password_length * 2)

password = ""
for i in 0 .. password_length:
    pair  = expanded[i*2 .. i*2+2]
    index = u16_from_le_bytes(pair) % len(charset)
    password += charset[index]
```

Instantiated as:
`blake3::Hasher::new_derive_key(context).update(site_key).finalize_xof()`

### Charsets

| Name          | Characters | Length |
|---|---|---|
| `alpha`       | `a-z A-Z` | 52 |
| `alphanumeric`| `a-z A-Z 0-9` | 62 |
| `full`        | `a-z A-Z 0-9 !@#$%^&*()-_=+[]{}|;:,.<>?` | 95 |
| `pin`         | `0-9` | 10 |
| `hex`         | `0-9 a-f` | 16 |

Character order within each charset is fixed in the source and must not change.

Maximum bias from modular reduction: `256 % 95 / 256 ≈ 0.4%` (negligible).

---

## Verifier

```
verifier = hex( site_key[0] ) || hex( site_key[1] )   // 4 hex chars
```

Example: if `site_key[0] = 0xa3` and `site_key[1] = 0xf9`, verifier = `"a3f9"`.

The verifier is **not secret**. It lets you confirm the correct counter value
during recovery without revealing any password material.

---

## Recovery (counter scanning)

When the rotation log is lost, iterate `n = 1, 2, 3, …` until
`verifier(site_key(master, site, user, n))` matches the stored verifier.
The correct `n` is the counter to use.

---

## Security Notes

- The seed phrase is the only secret. All other metadata (site names,
  usernames, counters, verifiers) may be stored or transmitted in plaintext.
- The master key is never stored. It is derived on demand and held in
  locked memory for the duration of the process.
- Passwords are never stored. They are derived on demand and optionally
  copied to the clipboard with a configurable auto-clear timeout.
- Passphrase adds a second factor. Losing the passphrase with only the
  mnemonic means all passwords are unrecoverable.
- scrypt's memory-hardness (128 MiB per attempt) significantly raises the
  cost of brute-forcing a weak passphrase compared to a fast KDF.
