#!/usr/bin/env sh
# build.sh — compile hdpassw (run as your normal user, not root)
#
# DEPENDENCIES
#   Rust toolchain — install from https://rustup.rs
#   On Debian/Ubuntu: sudo apt install build-essential pkg-config libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
#   On Fedora/RHEL:   sudo dnf install gcc pkg-config libxcb-devel
#   On Arch Linux:    sudo pacman -S base-devel libxcb
#   On macOS:         xcode-select --install  (no extra deps needed)
#
# USAGE
#   ./build.sh [OPTIONS]
#
# OPTIONS
#   --gui    Also build the optional GUI (hdpassw-gui)
#
# Then install with:
#   sudo ./install.sh
#
# ENVIRONMENT
#   CARGO    Override the cargo binary path.

set -eu

# ── Argument parsing ──────────────────────────────────────────────────────────
GUI=0
while [ $# -gt 0 ]; do
    case "$1" in
        --gui) GUI=1; shift ;;
        -h|--help)
            sed -n '/^# USAGE/,/^[^#]/{ /^[^#]/d; s/^# \{0,1\}//; p }' "$0"
            exit 0
            ;;
        *) printf "Unknown argument: %s  (try --help)\n" "$1" >&2; exit 1 ;;
    esac
done

# ── Colours ───────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    GREEN='\033[0;32m'; RED='\033[0;31m'; BOLD='\033[1m'; RESET='\033[0m'
else
    GREEN=''; RED=''; BOLD=''; RESET=''
fi

info()  { printf "${GREEN}  ✓${RESET}  %s\n" "$*"; }
error() { printf "${RED}  ✗${RESET}  %s\n" "$*" >&2; exit 1; }
step()  { printf "\n${BOLD}%s${RESET}\n" "$*"; }

# ── Must not run as root ───────────────────────────────────────────────────────
[ "$(id -u)" -ne 0 ] \
    || error "Do not run build.sh as root — rustup toolchains are user-scoped.
  Run as your normal user:  ./build.sh"

# ── Source directory check ────────────────────────────────────────────────────
[ -f Cargo.toml ] \
    || error "Cargo.toml not found — run this script from the hdpassw source directory."
grep -q 'name.*=.*"hdpassw"' Cargo.toml \
    || error "This doesn't look like the hdpassw source directory."
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

# ── Find cargo ────────────────────────────────────────────────────────────────
# Try candidates in order; stop at the first one where --version succeeds.
CARGO_BIN=""
CARGO_VERSION=""

_cargo_ver() {
    # Run "$@" --version and return the version string, or fail.
    _v=$("$@" --version 2>/dev/null | grep -o '[0-9]*\.[0-9]*\.[0-9]*' | head -1)
    [ -n "$_v" ] && printf '%s' "$_v"
}

if [ -n "${CARGO:-}" ]; then
    # Explicit override — honour it and fail loudly if it doesn't work.
    CARGO_BIN="$CARGO"
    CARGO_VERSION=$(_cargo_ver "$CARGO_BIN") \
        || error "CARGO='${CARGO}' failed to run."

elif CARGO_VERSION=$(_cargo_ver "${HOME}/.cargo/bin/cargo" 2>/dev/null); then
    CARGO_BIN="${HOME}/.cargo/bin/cargo"

elif command -v rustup >/dev/null 2>&1 \
  && _rpath=$(rustup which cargo 2>/dev/null) && [ -n "$_rpath" ] \
  && CARGO_VERSION=$(_cargo_ver "$_rpath"); then
    CARGO_BIN="$_rpath"

elif command -v rustup >/dev/null 2>&1 \
  && CARGO_VERSION=$(rustup run stable cargo --version 2>/dev/null \
       | grep -o '[0-9]*\.[0-9]*\.[0-9]*' | head -1) && [ -n "$CARGO_VERSION" ]; then
    CARGO_BIN="rustup run stable cargo"

elif command -v cargo >/dev/null 2>&1 \
  && CARGO_VERSION=$(_cargo_ver cargo); then
    CARGO_BIN="cargo"

else
    # Last resort: find cargo directly inside any installed rustup toolchain.
    # Covers the case where rustup has a versioned toolchain (e.g. 1.89-x86_64)
    # but no 'stable' alias and no working proxy in ~/.cargo/bin.
    for _tc_cargo in "${HOME}/.rustup/toolchains"/*/bin/cargo; do
        [ -x "$_tc_cargo" ] || continue
        CARGO_VERSION=$(_cargo_ver "$_tc_cargo") || continue
        CARGO_BIN="$_tc_cargo"
        # Tell rustup proxies (rustc, etc.) to use this same toolchain,
        # so cargo can find rustc without a default toolchain being set.
        _tc_name=$(basename "$(dirname "$(dirname "$_tc_cargo")")")
        export RUSTUP_TOOLCHAIN="$_tc_name"
        break
    done
fi

if [ -z "$CARGO_BIN" ]; then
    # Nothing worked — print diagnostics and exit.
    printf "\n" >&2
    command -v rustup >/dev/null 2>&1 \
        && printf "  rustup toolchains: %s\n" "$(rustup toolchain list 2>&1)" >&2
    error "cargo not found or not working.
  Install the Rust toolchain from https://rustup.rs :
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  Or install a toolchain via rustup:
    rustup toolchain install stable"
fi

# ── Build ─────────────────────────────────────────────────────────────────────
step "Checking prerequisites"
info "cargo ${CARGO_VERSION}"
info "hdpassw ${VERSION}"

step "Building release binary"
# CARGO_BIN may be a multi-word command (e.g. "rustup run stable cargo")
# so we use sh -c to invoke it correctly.
sh -c "$CARGO_BIN build --release"
info "Built: target/release/hdpassw"

if [ "$GUI" -eq 1 ]; then
    step "Building GUI (hdpassw-gui)"
    sh -c "$CARGO_BIN build --release --features gui --bin hdpassw-gui"
    info "Built: target/release/hdpassw-gui"
fi

printf "\n${BOLD}Build complete.${RESET}\n"
printf "  Now install as root:\n"
if [ "$GUI" -eq 1 ]; then
    printf "    ${BOLD}sudo ./install.sh --gui${RESET}\n\n"
else
    printf "    ${BOLD}sudo ./install.sh${RESET}\n\n"
fi
