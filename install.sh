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

# Use the local checkout when this script is run from one; otherwise (piped,
# or copied elsewhere) clone a fresh copy to build.
if [ -f "$0" ] && [ -f "$(dirname "$0")/Cargo.toml" ]; then
    cd "$(dirname "$0")"
else
    if ! command -v git >/dev/null 2>&1; then
        echo "install.sh: git not found — clone the repo manually instead: https://github.com/adiraj66132/ranger-rs" >&2
        exit 1
    fi
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    echo "cloning ranger-rs..."
    git clone --depth 1 https://github.com/adiraj66132/ranger-rs.git "$tmp"
    cd "$tmp"
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
