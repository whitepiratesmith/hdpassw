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

You can get started entirely from the CLI, entirely from the GUI, or mix
the two — both read and write the same encrypted vault and metadata file.

```bash
# 1. Generate a seed phrase — write it down offline
hdpassw init          # or: hdpassw seed new

# 2. Register a site (metadata only, no passwords stored)
hdpassw add github.com --user jakob

# 3. Generate a password (copied to clipboard, cleared after 30s)
hdpassw gen github.com

# 4. List all sites
hdpassw ls
```

Or skip the CLI entirely: launch `hdpassw-gui` (see [GUI](#gui-optional)
below) and its first-run wizard will walk you through generating or
restoring a seed phrase, right from the window.

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

No encrypted vault yet? `hdpassw-gui` doesn't require the CLI to set one up —
launching it with no vault present shows a first-run wizard that generates a
new seed phrase (with a short recall quiz to catch transcription mistakes)
or restores an existing one, then saves the vault, all from the window. The
CLI and GUI are equally capable starting points and share the same vault and
site metadata.

## Usage

```
hdpassw <COMMAND>

Commands:
  init     Get started: create a new seed phrase or restore an existing one
  gen      Generate (and copy) the password for a site
  add      Add or update a site record in the metadata file
  ls       List all known sites
  rm       Remove a site record from the metadata file
  seed     Seed phrase management (new, check, restore, remove)
  export   Export metadata to stdout (no passwords)
  rotate   Bump the global rotation counter and list sites to regenerate
  bump     Bump the rotation counter for a single saved site
  recover  Recover counter by scanning from a seed alone
  pwned    Check site password(s) against Have I Been Pwned
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

### Change a site's password (catch up to the current rotation level)

`hdpassw bump` walks you through an actual password change, not just a
counter bump: it derives the OLD password (to log in) and the NEW
password (to set), hands them to you in turn via the clipboard, and only
updates the metadata file once that hand-off is done — so a cancelled
run leaves your metadata untouched.

```bash
# After hdpassw rotate, walk through the change for one site
hdpassw bump github.com
# Step 1/2: copies the OLD password — log in and start the change
# Step 2/2: copies the NEW password — paste it as the new one
# Then confirms before saving the new counter to metadata

# Set an exact counter value instead of catching up to the global level
hdpassw bump github.com --to 5

# Headless/SSH: print both passwords instead of using the clipboard
hdpassw bump github.com --reveal

# Scripting: get both passwords as JSON; --yes persists the new counter
hdpassw bump github.com --json --yes
```

`hdpassw rotate` (CLI) and the "Rotate" button (GUI) bump only the
*global* rotation counter and list which sites have fallen behind — they
never touch a site's own counter or password. `hdpassw ls` shows the
global counter and flags stale sites; the GUI shows the same counter
next to the site list, with a "Bump" button that walks through the
password hand-off on each site that's behind.

### Recover from seed only

```bash
# Lost the metadata file? Use the verifier you wrote down
hdpassw recover github.com --user jakob --verifier a3f9
# → Found! counter = 2
```

### Check for known breaches

```bash
# Check one site
hdpassw pwned github.com

# Check every site in the metadata file
hdpassw pwned
```

Uses the [Have I Been Pwned](https://haveibeenpwned.com/API/v3#PwnedPasswords)
"Pwned Passwords" range API with k-anonymity: only the first 5 characters of
the password's SHA-1 hash are sent over the network — the password itself,
and its full hash, never leave your machine. No API key needed. This is the
only command in hdpassw that makes a network request.

The GUI's "Test if pwned" button does the same check for every stored site
in one click.

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
