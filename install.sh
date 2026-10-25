#!/bin/sh
# Install the latest prebuilt ranger-rs release.
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

case "$(uname -s):$(uname -m)" in
    Linux:x86_64|Linux:amd64)  target=x86_64-unknown-linux-gnu ;;
    Linux:aarch64|Linux:arm64) target=aarch64-unknown-linux-gnu ;;
    Darwin:x86_64)             target=x86_64-apple-darwin ;;
    Darwin:arm64)              target=aarch64-apple-darwin ;;
    *) echo "install.sh: unsupported platform: $(uname -s) $(uname -m)" >&2; exit 1 ;;
esac

if ! command -v curl >/dev/null 2>&1; then
    echo "install.sh: curl not found" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

url="https://github.com/adiraj66132/ranger-rs/releases/latest/download/ranger-rs-$target.tar.gz"
echo "downloading $url"
curl -fsSL "$url" -o "$tmp/ranger-rs.tar.gz"
tar -xzf "$tmp/ranger-rs.tar.gz" -C "$tmp"

$sudo install -d "$prefix"
$sudo install -m755 "$tmp/ranger-rs" "$prefix/ranger-rs"
$sudo ln -sf ranger-rs "$prefix/rr"
echo "installed ranger-rs and rr to $prefix"

case ":$PATH:" in
    *":$prefix:"*) ;;
    *) echo "note: $prefix is not on PATH; add this to your shell config:"
       echo "  export PATH=\"$prefix:\$PATH\"" ;;
esac
