# hdpassw — Makefile
#
# DEPENDENCIES
#   Rust toolchain (https://rustup.rs) — required to build
#   Debian/Ubuntu: sudo apt install build-essential pkg-config \
#                    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
#   Fedora/RHEL:   sudo dnf install gcc pkg-config libxcb-devel
#   Arch Linux:    sudo pacman -S base-devel libxcb
#   macOS:         xcode-select --install  (no extra deps)
#
# TARGETS
#   make              Build the release binary (as your regular user)
#   make install      Build + install to PREFIX (default: /usr/local)
#   make install-only Install a pre-built binary — skips cargo (safe under sudo)
#   make uninstall    Remove everything installed
#   make test         Run all tests
#   make check        clippy + fmt check
#   make fix          Auto-format and apply clippy fixes
#   make audit        cargo-audit (must be installed separately)
#   make dist         Build a release tarball + sha256 checksum
#   make clean        Remove build artefacts
#
# RECOMMENDED WORKFLOW for system-wide install (/usr/local):
#
#   make                          # build as your regular user
#   sudo make install-only        # install files as root — no cargo needed
#
# VARIABLES
#   PREFIX   installation root          (default: /usr/local)
#   DESTDIR  staging root for packagers (default: empty)
#   CARGO    path to cargo              (default: auto-detected)

PREFIX  ?= /usr/local
DESTDIR ?=

# Auto-detect cargo: prefer ~/.cargo/bin/cargo so sudo installs work correctly
# when the real user's rustup toolchain is what we want.
CARGO ?= $(or \
    $(wildcard $(HOME)/.cargo/bin/cargo), \
    $(shell command -v cargo 2>/dev/null), \
    cargo)

# ── Derived paths ─────────────────────────────────────────────────────────────
BINDIR       := $(DESTDIR)$(PREFIX)/bin
MANDIR       := $(DESTDIR)$(PREFIX)/share/man/man1
BASH_COMPDIR := $(DESTDIR)$(PREFIX)/share/bash-completion/completions
ZSH_COMPDIR  := $(DESTDIR)$(PREFIX)/share/zsh/site-functions
FISH_COMPDIR := $(DESTDIR)$(PREFIX)/share/fish/vendor_completions.d

TARGET_BIN := target/release/hdpassw
VERSION    := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

# ── Default target ────────────────────────────────────────────────────────────
.PHONY: all
all: build

# ── Build ─────────────────────────────────────────────────────────────────────
.PHONY: build
build: $(TARGET_BIN)

$(TARGET_BIN): $(shell find src -name '*.rs') Cargo.toml
	$(CARGO) build --release

# ── Install (build + copy) ────────────────────────────────────────────────────
.PHONY: install
install: build install-only

# ── Install-only (copy pre-built binary — safe under sudo) ───────────────────
.PHONY: install-only
install-only:
	@test -f $(TARGET_BIN) || { \
	    echo ""; \
	    echo "  ✗  $(TARGET_BIN) not found."; \
	    echo "     Build it first as your regular user:  make build"; \
	    echo ""; \
	    exit 1; \
	}
	@echo "Installing hdpassw $(VERSION) to $(DESTDIR)$(PREFIX)"

	install -d $(BINDIR)
	install -m 755 $(TARGET_BIN) $(BINDIR)/hdpassw

	install -d $(MANDIR)
	install -m 644 man/hdpassw.1 $(MANDIR)/hdpassw.1

	install -d $(BASH_COMPDIR)
	install -m 644 completions/hdpassw.bash $(BASH_COMPDIR)/hdpassw

	install -d $(ZSH_COMPDIR)
	install -m 644 completions/hdpassw.zsh $(ZSH_COMPDIR)/_hdpassw

	install -d $(FISH_COMPDIR)
	install -m 644 completions/hdpassw.fish $(FISH_COMPDIR)/hdpassw.fish

	@echo ""
	@echo "  ✓  hdpassw installed to $(BINDIR)/hdpassw"
	@echo ""
	@echo "  Shell completions:"
	@echo "    bash  →  source $(BASH_COMPDIR)/hdpassw"
	@echo "    zsh   →  fpath=($(ZSH_COMPDIR) \$$fpath) && compinit"
	@echo "    fish  →  (auto-loaded)"
	@echo ""
	@echo "  Man page:  man hdpassw"
	@echo ""

# ── Uninstall ─────────────────────────────────────────────────────────────────
.PHONY: uninstall
uninstall:
	@echo "Removing hdpassw from $(DESTDIR)$(PREFIX)"
	rm -f $(BINDIR)/hdpassw
	rm -f $(MANDIR)/hdpassw.1
	rm -f $(BASH_COMPDIR)/hdpassw
	rm -f $(ZSH_COMPDIR)/_hdpassw
	rm -f $(FISH_COMPDIR)/hdpassw.fish
	@echo "  ✓  Uninstalled"
	@echo "     (metadata file ~/.config/hdpassw/sites.toml was not removed)"

# ── Test ──────────────────────────────────────────────────────────────────────
.PHONY: test
test:
	$(CARGO) test --all

# ── Check (lint + format) ─────────────────────────────────────────────────────
.PHONY: check
check:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets -- -D warnings

# ── Fix (auto-format + clippy fixes) ─────────────────────────────────────────
.PHONY: fix
fix:
	$(CARGO) fmt
	$(CARGO) clippy --fix --allow-dirty

# ── Security audit ────────────────────────────────────────────────────────────
.PHONY: audit
audit:
	@command -v cargo-audit >/dev/null 2>&1 || { \
	    echo "cargo-audit not installed.  Run:  cargo install cargo-audit"; \
	    exit 1; \
	}
	$(CARGO) audit

# ── Clean ─────────────────────────────────────────────────────────────────────
.PHONY: clean
clean:
	$(CARGO) clean

# ── Distribution tarball ──────────────────────────────────────────────────────
.PHONY: dist
dist: build
	$(eval DIST_NAME := hdpassw-$(VERSION)-$(shell uname -m)-$(shell uname -s | tr '[:upper:]' '[:lower:]'))
	$(eval DIST_DIR  := /tmp/$(DIST_NAME))
	rm -rf $(DIST_DIR)
	mkdir -p $(DIST_DIR)
	cp $(TARGET_BIN)             $(DIST_DIR)/hdpassw
	cp README.md ALGORITHM.md    $(DIST_DIR)/
	cp SECURITY.md CHANGELOG.md  $(DIST_DIR)/
	cp -r completions man        $(DIST_DIR)/
	cp Makefile install.sh       $(DIST_DIR)/
	tar -czf $(DIST_NAME).tar.gz -C /tmp $(DIST_NAME)
	sha256sum $(DIST_NAME).tar.gz > $(DIST_NAME).tar.gz.sha256
	rm -rf $(DIST_DIR)
	@echo "  ✓  $(DIST_NAME).tar.gz"
	@echo "  ✓  $(DIST_NAME).tar.gz.sha256"

# ── Help ──────────────────────────────────────────────────────────────────────
.PHONY: help
help:
	@echo ""
	@echo "hdpassw $(VERSION)"
	@echo ""
	@echo "  Targets:"
	@echo "    make              Build release binary"
	@echo "    make install      Build + install (binary, man page, completions)"
	@echo "    make install-only Install pre-built binary — safe under sudo"
	@echo "    make uninstall    Remove installed files"
	@echo "    make test         Run all tests"
	@echo "    make check        clippy + fmt check"
	@echo "    make fix          Auto-format and apply clippy fixes"
	@echo "    make audit        Dependency vulnerability scan (cargo-audit)"
	@echo "    make dist         Release tarball + sha256"
	@echo "    make clean        Remove build artefacts"
	@echo ""
	@echo "  Variables:"
	@echo "    PREFIX=$(PREFIX)"
	@echo "    DESTDIR=$(DESTDIR)"
	@echo "    CARGO=$(CARGO)"
	@echo ""
	@echo "  System-wide install (recommended):"
	@echo "    make                    # as your user"
	@echo "    sudo make install-only  # as root"
	@echo ""
	@echo "  User-local install (no sudo):"
	@echo "    make install PREFIX=~/.local"
	@echo ""
	@echo "  Packager (DESTDIR staging):"
	@echo "    make install DESTDIR=/tmp/pkg PREFIX=/usr"
	@echo ""
