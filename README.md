# hdpassw

Deterministic password manager backed by a single BIP39 seed phrase.

Every password is derived on the fly — nothing is stored. Back up 24 words; recover everything.

Built for cryptography enthusiasts — no blockchain, no coins, just strong deterministic key derivation.

**https://hdpassw.com**

## How it works

```
seed phrase  →  master key   →  site key  →  password
                (scrypt)        (BLAKE3)      (BLAKE3 XOF)
```

See [ALGORITHM.md](ALGORITHM.md) for the full cryptographic specification.

## Quick start

```bash
# 1. Generate a seed phrase — write it down offline
hdpassw seed new

# 2. Register a site (metadata only, no passwords stored)
hdpassw add github.com --user jakob

# 3. Generate a password (copied to clipboard, cleared after 30s)
hdpassw gen github.com

# 4. List all sites
hdpassw ls
```

## Dependencies

You need a Rust toolchain. Install it from **https://rustup.rs** — one command,
no root required:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Linux, the clipboard crate also needs a few X11/Wayland libraries:

```bash
# Debian / Ubuntu
sudo apt install build-essential pkg-config \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora / RHEL
sudo dnf install gcc pkg-config libxcb-devel

# Arch Linux
sudo pacman -S base-devel libxcb

# macOS — no extra deps (uses native clipboard APIs)
xcode-select --install
```

## Installation

### From source — user-local (no sudo)

```bash
git clone https://github.com/whitepiratesmith/hdpassw
cd hdpassw
./install.sh --prefix ~/.local
# add ~/.local/bin to PATH if not already there
```

### From source — system-wide (/usr/local)

Build as your regular user first, **then** install as root.
This avoids running `cargo` under `sudo`, which breaks rustup.

```bash
git clone https://github.com/whitepiratesmith/hdpassw
cd hdpassw

# Step 1 — build (as your user, no sudo)
make

# Step 2 — install files (as root, no cargo needed)
sudo make install-only
```

Or with the shell script:

```bash
./install.sh --build-only          # step 1: build as your user
sudo ./install.sh --skip-build     # step 2: install as root
```

## GUI (optional)

A minimal desktop GUI (`hdpassw-gui`) is available as an opt-in build, built
with [egui](https://github.com/emilk/egui) — pure Rust, no webview. It unlocks
the encrypted seed vault, lists known sites, and lets you generate/copy
passwords or add new sites.

```bash
# Build
make gui                  # or: ./build.sh --gui

# Install (also adds a desktop launcher + icon)
sudo make install-gui     # or: sudo ./install.sh --gui
```

The CLI remains the primary interface; the GUI requires an existing encrypted
vault (`hdpassw seed new` / `hdpassw seed restore`).

## Usage

```
hdpassw <COMMAND>

Commands:
  gen      Generate (and copy) the password for a site
  add      Add or update a site record in the metadata file
  ls       List all known sites
  rm       Remove a site record from the metadata file
  seed     Seed phrase management (new, check)
  export   Export metadata to stdout (no passwords)
  recover  Recover counter by scanning from a seed alone
```

### Generate a password

```bash
# Uses settings from metadata file
hdpassw gen github.com

# Ad-hoc, no metadata needed
hdpassw gen github.com --user jakob --length 32 --charset full

# Print instead of clipboard (careful!)
hdpassw gen github.com --reveal

# JSON output (for scripting — password omitted by default)
hdpassw gen github.com --json
```

### Add / update a site

```bash
hdpassw add github.com --user jakob --length 32 --charset full
hdpassw add netflix.com --user jakobhusu@gmail.com
```

### Rotate a password

```bash
# Increment the counter for one site; all others are unchanged
hdpassw add github.com --user jakob --counter 2
hdpassw gen github.com   # now generates password for n=2
```

### Recover from seed only

```bash
# Lost the metadata file? Use the verifier you wrote down
hdpassw recover github.com --user jakob --verifier a3f9
# → Found! counter = 2
```

### Scripting

```bash
# Passphrase from environment (never pass as CLI arg)
export HDPASSW_PASSPHRASE="my passphrase"
hdpassw gen github.com --reveal

# Mnemonic from environment (for CI/testing only — warn is printed)
export HDPASSW_MNEMONIC="word1 word2 ... word24"
hdpassw gen github.com --reveal
```

## Backup strategy

| What | Sensitivity | Where to store |
|---|---|---|
| 24-word seed phrase | 🔴 Secret | Paper/metal, offline safe |
| Optional passphrase | 🔴 Secret | Memorised — never written |
| `~/.config/hdpassw/sites.toml` | 🟢 Not secret | Git, cloud, anywhere |

The seed phrase is the **only** thing that must be protected.
The metadata file is safe to store anywhere — it contains no passwords.

## Security

See [SECURITY.md](SECURITY.md) for the full threat model and vulnerability
reporting policy.

Key properties:
- Passwords are never stored
- Master key is held in locked memory and zeroed on drop
- Passphrase is read via `rpassword` (never echoed, never in process args)
- Clipboard is auto-cleared after 30 seconds (configurable)
- Metadata file is `chmod 600` on Unix
- No custom cryptography — all primitives from audited Rust crates

## License

BSD-2-Clause
