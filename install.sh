#!/usr/bin/env sh
# install.sh — install a pre-built hdpassw binary (run as root)
#
# Build the binary first with:
#   ./build.sh
#
# Then install:
#   sudo ./install.sh
#
# USAGE
#   sudo ./install.sh [OPTIONS]
#
# OPTIONS
#   --prefix PATH     Installation prefix (default: /usr/local)
#   --gui             Also install the optional GUI (hdpassw-gui)
#   --uninstall       Remove a previous installation
#   --no-completions  Skip shell completions
#   --no-man          Skip man page
#   -h, --help        Show this help

set -eu

# ── Defaults ──────────────────────────────────────────────────────────────────
PREFIX="/usr/local"
UNINSTALL=0
COMPLETIONS=1
MAN=1
GUI=0

# ── Colours ───────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'
    BOLD='\033[1m'; RESET='\033[0m'
else
    GREEN=''; YELLOW=''; RED=''; BOLD=''; RESET=''
fi

info()  { printf "${GREEN}  ✓${RESET}  %s\n" "$*"; }
warn()  { printf "${YELLOW}  !${RESET}  %s\n" "$*"; }
error() { printf "${RED}  ✗${RESET}  %s\n" "$*" >&2; exit 1; }
step()  { printf "\n${BOLD}%s${RESET}\n" "$*"; }

# ── Argument parsing ──────────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)        PREFIX="${2:?--prefix requires a value}"; shift 2 ;;
        --prefix=*)      PREFIX="${1#--prefix=}"; shift ;;
        --gui)           GUI=1;          shift ;;
        --uninstall)     UNINSTALL=1;    shift ;;
        --no-completions) COMPLETIONS=0; shift ;;
        --no-man)        MAN=0;          shift ;;
        -h|--help)
            sed -n '/^# USAGE/,/^[^#]/{ /^[^#]/d; s/^# \{0,1\}//; p }' "$0"
            exit 0
            ;;
        *) error "Unknown argument: $1  (try --help)" ;;
    esac
done

# ── Derived paths ─────────────────────────────────────────────────────────────
BINDIR="${PREFIX}/bin"
MANDIR="${PREFIX}/share/man/man1"
BASH_COMPDIR="${PREFIX}/share/bash-completion/completions"
ZSH_COMPDIR="${PREFIX}/share/zsh/site-functions"
FISH_COMPDIR="${PREFIX}/share/fish/vendor_completions.d"
DESKTOPDIR="${PREFIX}/share/applications"
ICONDIR="${PREFIX}/share/icons/hicolor/scalable/apps"
TARGET_BIN="target/release/hdpassw"
TARGET_GUI_BIN="target/release/hdpassw-gui"

# ── Source directory check ────────────────────────────────────────────────────
[ -f Cargo.toml ] \
    || error "Cargo.toml not found — run this script from the hdpassw source directory."
grep -q 'name.*=.*"hdpassw"' Cargo.toml \
    || error "This doesn't look like the hdpassw source directory."
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

# ── Uninstall ─────────────────────────────────────────────────────────────────
if [ "$UNINSTALL" -eq 1 ]; then
    step "Uninstalling hdpassw from ${PREFIX}"
    rm -f "${BINDIR}/hdpassw"            && info "Removed ${BINDIR}/hdpassw"
    rm -f "${BINDIR}/hdpassw-gui"        && info "Removed ${BINDIR}/hdpassw-gui"
    rm -f "${MANDIR}/hdpassw.1"          && info "Removed ${MANDIR}/hdpassw.1"
    rm -f "${BASH_COMPDIR}/hdpassw"      && info "Removed bash completion"
    rm -f "${ZSH_COMPDIR}/_hdpassw"      && info "Removed zsh completion"
    rm -f "${FISH_COMPDIR}/hdpassw.fish" && info "Removed fish completion"
    rm -f "${ICONDIR}/hdpassw-gui.svg"   && info "Removed desktop icon"
    rm -f "${DESKTOPDIR}/hdpassw-gui.desktop" && info "Removed desktop launcher"
    printf "\n"
    warn "Your data file (~/.config/hdpassw/sites.toml) was not removed."
    printf "\n"
    exit 0
fi

# ── Check pre-built binary exists ─────────────────────────────────────────────
step "Checking prerequisites"
[ -f "${TARGET_BIN}" ] \
    || error "No binary found at ${TARGET_BIN}.
  Build it first (as your normal user):
    ./build.sh"
info "hdpassw ${VERSION}  (${TARGET_BIN})"

if [ "$GUI" -eq 1 ]; then
    [ -f "${TARGET_GUI_BIN}" ] \
        || error "No binary found at ${TARGET_GUI_BIN}.
  Build it first (as your normal user):
    ./build.sh --gui"
    info "hdpassw-gui ${VERSION}  (${TARGET_GUI_BIN})"
fi

# ── Install binary ────────────────────────────────────────────────────────────
step "Installing to ${PREFIX}"

mkdir -p "${BINDIR}"
cp "${TARGET_BIN}" "${BINDIR}/hdpassw"
chmod 755 "${BINDIR}/hdpassw"
info "Binary: ${BINDIR}/hdpassw"

if [ "$GUI" -eq 1 ]; then
    cp "${TARGET_GUI_BIN}" "${BINDIR}/hdpassw-gui"
    chmod 755 "${BINDIR}/hdpassw-gui"
    info "Binary: ${BINDIR}/hdpassw-gui"

    mkdir -p "${ICONDIR}"
    cp assets/hdpassw-gui.svg "${ICONDIR}/hdpassw-gui.svg"
    chmod 644 "${ICONDIR}/hdpassw-gui.svg"
    info "Icon: ${ICONDIR}/hdpassw-gui.svg"

    mkdir -p "${DESKTOPDIR}"
    cp assets/hdpassw-gui.desktop "${DESKTOPDIR}/hdpassw-gui.desktop"
    chmod 644 "${DESKTOPDIR}/hdpassw-gui.desktop"
    info "Desktop launcher: ${DESKTOPDIR}/hdpassw-gui.desktop"
fi

# ── Install man page ──────────────────────────────────────────────────────────
if [ "$MAN" -eq 1 ] && [ -f man/hdpassw.1 ]; then
    mkdir -p "${MANDIR}"
    cp man/hdpassw.1 "${MANDIR}/hdpassw.1"
    chmod 644 "${MANDIR}/hdpassw.1"
    info "Man page: ${MANDIR}/hdpassw.1"
fi

# ── Install shell completions ─────────────────────────────────────────────────
if [ "$COMPLETIONS" -eq 1 ]; then
    if [ -f completions/hdpassw.bash ]; then
        mkdir -p "${BASH_COMPDIR}"
        cp completions/hdpassw.bash "${BASH_COMPDIR}/hdpassw"
        chmod 644 "${BASH_COMPDIR}/hdpassw"
        info "Bash completion: ${BASH_COMPDIR}/hdpassw"
    fi
    if [ -f completions/hdpassw.zsh ]; then
        mkdir -p "${ZSH_COMPDIR}"
        cp completions/hdpassw.zsh "${ZSH_COMPDIR}/_hdpassw"
        chmod 644 "${ZSH_COMPDIR}/_hdpassw"
        info "Zsh completion: ${ZSH_COMPDIR}/_hdpassw"
    fi
    if [ -f completions/hdpassw.fish ]; then
        mkdir -p "${FISH_COMPDIR}"
        cp completions/hdpassw.fish "${FISH_COMPDIR}/hdpassw.fish"
        chmod 644 "${FISH_COMPDIR}/hdpassw.fish"
        info "Fish completion: ${FISH_COMPDIR}/hdpassw.fish"
    fi
fi

# ── Refresh desktop databases (GUI only) ─────────────────────────────────────
if [ "$GUI" -eq 1 ]; then
    command -v update-desktop-database >/dev/null 2>&1 \
        && update-desktop-database "${DESKTOPDIR}" >/dev/null 2>&1 || true
    command -v gtk-update-icon-cache >/dev/null 2>&1 \
        && gtk-update-icon-cache "${PREFIX}/share/icons/hicolor" >/dev/null 2>&1 || true
fi

# ── Done ──────────────────────────────────────────────────────────────────────
printf "\n${BOLD}Installation complete.${RESET}\n\n"

case ":${PATH}:" in
    *":${BINDIR}:"*) info "${BINDIR} is on your PATH" ;;
    *)
        warn "${BINDIR} is not in PATH."
        printf "       Add to your shell profile:\n"
        printf "         ${BOLD}export PATH=\"${BINDIR}:\$PATH\"${RESET}\n\n"
        ;;
esac

printf "  Run ${BOLD}hdpassw --help${RESET} to get started.\n"
printf "  Run ${BOLD}hdpassw seed new${RESET} to generate your seed phrase.\n\n"

if [ "$GUI" -eq 1 ]; then
    printf "  hdpassw-gui is available in your application menu as \"hdpassw\".\n\n"
fi

if [ "$COMPLETIONS" -eq 1 ]; then
    printf "  Shell completions:\n"
    printf "    bash  source ${BASH_COMPDIR}/hdpassw\n"
    printf "    zsh   fpath=(${ZSH_COMPDIR} \$fpath) && compinit\n"
    printf "    fish  auto-loaded\n\n"
fi
