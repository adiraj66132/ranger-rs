#!/bin/sh
# Build ranger-rs from this checkout and install the binary.
#
#   ./install.sh                 install to ~/.local/bin
#   PREFIX=/opt/bin ./install.sh install to a custom prefix
#   ./install.sh --system        install to /usr/local/bin (uses sudo)
set -eu

prefix="${PREFIX:-$HOME/.local/bin}"
sudo=""

for arg in "$@"; do
    case "$arg" in
        --system) prefix="/usr/local/bin"; sudo="sudo" ;;
        -h|--help) sed -n '2,6p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "install.sh: unknown option: $arg" >&2; exit 1 ;;
    esac
done

if ! command -v cargo >/dev/null 2>&1; then
    echo "install.sh: cargo not found. Install Rust first: https://rustup.rs" >&2
    exit 1
fi

cd "$(dirname "$0")"
if [ ! -f Cargo.toml ]; then
    echo "install.sh: Cargo.toml not found — run this from the ranger-rs checkout" >&2
    exit 1
fi

cargo build --release
$sudo install -d "$prefix"
$sudo install -m755 target/release/ranger-rs "$prefix/ranger-rs"
echo "installed ranger-rs to $prefix/ranger-rs"

case ":$PATH:" in
    *":$prefix:"*) ;;
    *) echo "note: $prefix is not on PATH; add this to your shell config:"
       echo "  export PATH=\"$prefix:\$PATH\"" ;;
esac
