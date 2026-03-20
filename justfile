set shell := ["bash", "-c"]

# Default target triple for the current machine
target := `rustc -vV | sed -n 's/^host: //p'`

# Release binary output directory
dist := "dist"

# ── Help ──────────────────────────────────────────────────────────────────────

[private]
default:
    @just --list

# ── Development ───────────────────────────────────────────────────────────────

# Build a debug binary
build:
    cargo build

# Build and run (pass args after --: `just run -- sync --help`)
run *args:
    cargo run -- {{ args }}

# Run all tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format source code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run lint + fmt-check (CI gate)
check: fmt-check lint
    cargo check

# ── Release ───────────────────────────────────────────────────────────────────

# Build a stripped, optimised release binary for the current host
release: _dist-dir
    cargo build --release --target {{ target }}
    just _strip-and-copy {{ target }}
    @echo "Binary → {{ dist }}/ghreposync-{{ target }}"

# Build a release binary for a specific target triple
release-target triple: _dist-dir
    cargo build --release --target {{ triple }}
    just _strip-and-copy {{ triple }}
    @echo "Binary → {{ dist }}/ghreposync-{{ triple }}"

# Strip and copy the binary into dist/
[private]
_strip-and-copy triple:
    #!/usr/bin/env bash
    set -euo pipefail
    src="target/{{ triple }}/release/ghreposync"
    dst="{{ dist }}/ghreposync-{{ triple }}"
    cp "$src" "$dst"
    # Strip debug symbols if strip is available
    if command -v strip &>/dev/null; then
        strip "$dst"
        echo "Stripped debug symbols"
    fi
    # UPX-compress if available (optional – further shrinks binary)
    if command -v upx &>/dev/null; then
        upx --best --lzma "$dst"
        echo "Compressed with UPX"
    fi
    ls -lh "$dst"

# Create the dist directory
[private]
_dist-dir:
    mkdir -p {{ dist }}

# Remove build artefacts and dist/
clean:
    cargo clean
    rm -rf {{ dist }}

# ── Info ──────────────────────────────────────────────────────────────────────

# Print current version from Cargo.toml
version:
    @cargo metadata --no-deps --format-version 1 | \
        python3 -c "import sys,json; d=json.load(sys.stdin); print(d['packages'][0]['version'])"
